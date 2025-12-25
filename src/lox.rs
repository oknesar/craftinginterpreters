mod token;

use std::io::Write;
use std::{env, fs, io, process};

pub struct Lox {
    has_error: bool,
}

impl Lox {
    pub fn start() {
        let mut lox = Lox { has_error: false };

        let args: Vec<String> = env::args().collect();
        match args.len() {
            0..3 => lox.run_prompt(),
            3 => lox.run_file(&args[2]),
            _ => {
                println!("cargo run -- path/to/**/*.lox")
            }
        }
    }

    fn run_prompt(&mut self) {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut content = String::new();
            io::stdin()
                .read_line(&mut content)
                .expect("Failed to read line");

            self.run(&content);

            if self.has_error == true {
                process::exit(65);
            }
        }
    }

    fn run_file(&mut self, file_path: &str) {
        let content = fs::read_to_string(file_path).unwrap();
        self.run(&content);
    }

    fn run(&mut self, code: &str) {
        print!("echo: {code}");
    }

    fn error(&mut self, line: i32, msg: &str) {
        self.report(line, "", msg);
    }

    fn report(&mut self, line: i32, info: &str, msg: &str) {
        self.has_error = true;
        println!("[line {line}] Error{info}: {msg}");
    }
}
