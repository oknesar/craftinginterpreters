use crate::lox::token::{Token, TokenKind};
use crate::lox::Reporter;

pub struct Scanner<'a, R: Reporter> {
    pub source: &'a str,
    pub tokens: Vec<Token<'a>>,
    reporter: &'a mut R,
    source_bytes: &'a [u8],
    start: usize,
    p: usize,
    line: u32,
}

impl<'a, R: Reporter> Scanner<'a, R> {
    pub fn new(reporter: &'a mut R, source: &'a str) -> Self {
        Self {
            source,
            source_bytes: source.as_bytes(),
            reporter,

            start: 0,
            p: 0,
            line: 0,
            tokens: vec![],
        }
    }

    pub fn scan(&mut self) {
        while !self.done() {
            self.start = self.p;
            self.parse_token();
        }

        self.start = self.p;
        self.add_token(TokenKind::EOF)
    }

    fn parse_token(&mut self) {
        let char = self.char_consume();
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
                if self.char_eq(b'/') {
                    self.comment()
                } else {
                    self.add_token(TokenKind::Slash)
                }
            }

            // One or two character tokens.
            b'!' => {
                if self.char_consume_if(b'=') {
                    self.add_token(TokenKind::BangEqual)
                } else {
                    self.add_token(TokenKind::Bang)
                }
            }
            b'=' => {
                if self.char_consume_if(b'=') {
                    self.add_token(TokenKind::EqualEqual)
                } else {
                    self.add_token(TokenKind::Equal)
                }
            }
            b'>' => {
                if self.char_consume_if(b'=') {
                    self.add_token(TokenKind::GreaterEqual)
                } else {
                    self.add_token(TokenKind::Greater)
                }
            }
            b'<' => {
                if self.char_consume_if(b'=') {
                    self.add_token(TokenKind::LessEqual)
                } else {
                    self.add_token(TokenKind::Less)
                }
            }

            b'"' => self.string(),
            _ => {
                panic!("Unexpected char '{char}'");
            }
        }
    }

    fn string(&mut self) {
        while !self.done() && !self.char_eq(b'"') {
            if self.char_eq(b'\n') {
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
        while !self.done() && !self.char_eq(b'\n') {
            self.step();
        }
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token {
            kind,
            line: self.line,
            lexeme: &self.source[self.start..self.p],
        })
    }

    fn char_consume_if(&mut self, char: u8) -> bool {
        if self.char_eq(char) {
            self.step();
            true
        } else {
            false
        }
    }

    fn char_consume(&mut self) -> &u8 {
        let char = &self.source_bytes[self.p];
        self.step();
        char
    }

    fn char_eq(&self, char: u8) -> bool {
        match self.source_bytes.get(self.p) {
            Some(current_char) => *current_char == char,
            None => false,
        }
    }

    fn step(&mut self) {
        self.p += 1;
    }

    fn done(&self) -> bool {
        return self.p >= self.source_bytes.len();
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
        let mut scanner = Scanner::new(&mut lox, "");

        scanner.scan();

        assert_eq!(
            scanner.tokens,
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
        ];

        for (code, kind) in variants {
            let mut lox = Lox { has_error: false };
            let mut scanner = Scanner::new(&mut lox, code);
            scanner.scan();

            assert_eq!(
                scanner.tokens,
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
        }
    }

    #[test]
    fn comment_only() {
        let mut lox = Lox { has_error: false };
        let mut scanner = Scanner::new(&mut lox, "// comment text");

        scanner.scan();

        assert_eq!(
            scanner.tokens,
            vec![Token {
                kind: TokenKind::EOF,
                line: 0,
                lexeme: ""
            }]
        );
    }
}
