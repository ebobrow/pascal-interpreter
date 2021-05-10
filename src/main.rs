mod interpreter;
mod lexer;
mod tokens;

use crate::interpreter::Interpreter;
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
        if input != "" {
            let lexer = Lexer::new(input);
            let mut interpreter = Interpreter::new(lexer);
            let result = interpreter.expr();
            println!("{}", result);
        }
    }
}
