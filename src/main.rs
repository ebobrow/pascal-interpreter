mod ast;
mod lexer;
mod tokens;

use crate::ast::{Interpreter, Parser};
use crate::lexer::Lexer;
use std::io::{stdin, stdout, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let mut input = String::new();
        print!("> ");
        stdout().flush()?;
        stdin().read_line(&mut input)?;
        if let Some('\n') = input.chars().next_back() {
            input.pop();
        }
        if input == "exit" {
            break Ok(());
        }
        if !input.is_empty() {
            let lexer = Lexer::new(input);
            let parser = Parser::new(lexer);
            let mut interpreter = Interpreter::new(parser);
            let result = interpreter.interpret();
            println!("{}", result);
        }
    }
}
