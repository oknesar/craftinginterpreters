use crate::lox::token::{Token, TokenKind};
use crate::lox::Reporter;

pub struct Scanner<'a, R>
where
    R: Reporter,
{
    pub source: &'a str,
    pub tokens: Vec<Token<'a>>,
    reporter: &'a mut R,
    source_bytes: &'a [u8],
    start: usize,
    pointer: usize,
    line: u32,
}

impl<'a, R> Scanner<'a, R>
where
    R: Reporter,
{
    pub fn scan(reporter: &'a mut R, source: &'a str) -> Vec<Token<'a>> {
        let mut scanner = Self {
            source,
            source_bytes: source.as_bytes(),
            reporter,
            start: 0,
            pointer: 0,
            line: 0,
            tokens: vec![],
        };

        while !scanner.done() {
            scanner.start = scanner.pointer;
            scanner.parse_token();
        }

        scanner.start = scanner.pointer;
        scanner.add_token(TokenKind::EOF);
        scanner.tokens
    }

    fn parse_token(&mut self) {
        let char = self.consume();
        match char {
            b' ' | b'\t' | b'\r' => (),
            b'\n' => {
                self.line += 1;
            }
            // Single-character tokens.
            b'(' => self.add_token(TokenKind::LeftParen),
            b')' => self.add_token(TokenKind::RightParen),
            b'{' => self.add_token(TokenKind::LeftBrace),
            b'}' => self.add_token(TokenKind::RightBrace),
            b',' => self.add_token(TokenKind::Comma),
            b'.' => self.add_token(TokenKind::Dot),
            b'-' => self.add_token(TokenKind::Minus),
            b'+' => self.add_token(TokenKind::Plus),
            b';' => self.add_token(TokenKind::Semicolon),
            b'*' => self.add_token(TokenKind::Star),
            b'/' => {
                // Maybe comment
                if self.char() == b'/' {
                    self.comment()
                } else {
                    self.add_token(TokenKind::Slash)
                }
            }
            // One or two character tokens.
            b'!' => {
                if self.consume_eq(b'=') {
                    self.add_token(TokenKind::BangEqual)
                } else {
                    self.add_token(TokenKind::Bang)
                }
            }
            b'=' => {
                if self.consume_eq(b'=') {
                    self.add_token(TokenKind::EqualEqual)
                } else {
                    self.add_token(TokenKind::Equal)
                }
            }
            b'>' => {
                if self.consume_eq(b'=') {
                    self.add_token(TokenKind::GreaterEqual)
                } else {
                    self.add_token(TokenKind::Greater)
                }
            }
            b'<' => {
                if self.consume_eq(b'=') {
                    self.add_token(TokenKind::LessEqual)
                } else {
                    self.add_token(TokenKind::Less)
                }
            }
            b'"' => self.string(),
            b'0'..=b'9' => self.number(),
            b'_' | b'a'..=b'z' | b'A'..=b'Z' => self.identifier(),
            _ => {
                let msg = format!("Unexpected character '{char}'.");
                self.reporter.error(self.line, &msg);
            }
        }
    }

    fn identifier(&mut self) {
        let char = self.char();
        while char.is_ascii_alphanumeric() || char == b'_' {
            self.step();
        }

        self.add_token(Self::get_identifier_kind(
            &self.source[self.start..self.pointer],
        ))
    }

    fn get_identifier_kind(lexeme: &str) -> TokenKind {
        match lexeme {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "fun" => TokenKind::Fun,
            "for" => TokenKind::For,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier,
        }
    }

    fn number(&mut self) {
        while self.char().is_ascii_digit() {
            self.step();
        }

        if self.char() == b'.' && self.next_char().is_ascii_digit() {
            self.step();
            self.step();
            while self.char().is_ascii_digit() {
                self.step();
            }
        }

        self.add_token(TokenKind::Number);
    }

    fn string(&mut self) {
        while !self.done() && self.char() != b'"' {
            if self.char() == b'\n' {
                self.line += 1;
            }
            self.step();
        }

        if self.done() {
            self.reporter.error(self.line, "Unterminated string.");
        } else {
            self.step();
            self.add_token(TokenKind::String);
        }
    }

    fn comment(&mut self) {
        while !self.done() && self.char() != b'\n' {
            self.step();
        }
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            line: self.line,
            lexeme: &self.source[self.start..self.pointer],
        })
    }

    fn consume_eq(&mut self, char: u8) -> bool {
        if self.char() == char {
            self.step();
            true
        } else {
            false
        }
    }

    fn consume(&mut self) -> u8 {
        let char = self.source_bytes[self.pointer];
        self.step();
        char
    }

    fn char(&self) -> u8 {
        if self.pointer < self.source_bytes.len() {
            self.source_bytes[self.pointer]
        } else {
            b'\0'
        }
    }

    fn next_char(&self) -> u8 {
        if self.pointer + 1 < self.source_bytes.len() {
            self.source_bytes[self.pointer + 1]
        } else {
            b'\0'
        }
    }

    fn step(&mut self) {
        self.pointer += 1;
    }

    fn done(&self) -> bool {
        self.pointer >= self.source_bytes.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lox::token::{Token, TokenKind};
    use crate::lox::Lox;

    #[test]
    fn empty_source() {
        let mut lox = Lox { has_error: false };
        let tokens = Scanner::scan(&mut lox, "");

        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::EOF,
                line: 0,
                lexeme: ""
            }]
        );
    }

    #[test]
    fn base_tokens() {
        let variants = [
            ("(", TokenKind::LeftParen),
            (")", TokenKind::RightParen),
            ("{", TokenKind::LeftBrace),
            ("}", TokenKind::RightBrace),
            (",", TokenKind::Comma),
            (".", TokenKind::Dot),
            ("-", TokenKind::Minus),
            ("+", TokenKind::Plus),
            (";", TokenKind::Semicolon),
            ("/", TokenKind::Slash),
            ("*", TokenKind::Star),
            ("*", TokenKind::Star),
            ("!", TokenKind::Bang),
            ("!=", TokenKind::BangEqual),
            ("=", TokenKind::Equal),
            ("==", TokenKind::EqualEqual),
            (">", TokenKind::Greater),
            (">=", TokenKind::GreaterEqual),
            ("<", TokenKind::Less),
            ("<=", TokenKind::LessEqual),
            ("\"string\"", TokenKind::String),
            ("123", TokenKind::Number),
            ("3.14", TokenKind::Number),
            ("and", TokenKind::And),
            ("class", TokenKind::Class),
            ("else", TokenKind::Else),
            ("false", TokenKind::False),
            ("fun", TokenKind::Fun),
            ("for", TokenKind::For),
            ("if", TokenKind::If),
            ("nil", TokenKind::Nil),
            ("or", TokenKind::Or),
            ("print", TokenKind::Print),
            ("return", TokenKind::Return),
            ("super", TokenKind::Super),
            ("this", TokenKind::This),
            ("true", TokenKind::True),
            ("var", TokenKind::Var),
            ("while", TokenKind::While),
            ("identifier", TokenKind::Identifier),
        ];

        for (code, kind) in variants {
            let mut lox = Lox { has_error: false };
            let tokens = Scanner::scan(&mut lox, code);

            assert_eq!(
                tokens,
                vec![
                    Token {
                        kind,
                        line: 0,
                        lexeme: code,
                    },
                    Token {
                        kind: TokenKind::EOF,
                        line: 0,
                        lexeme: "",
                    }
                ],
            );
            assert_eq!(lox.has_error, false);
        }
    }

    #[test]
    fn comment_only() {
        let mut lox = Lox { has_error: false };
        let tokens = Scanner::scan(&mut lox, "// comment text");

        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::EOF,
                line: 0,
                lexeme: ""
            }]
        );
    }
}
