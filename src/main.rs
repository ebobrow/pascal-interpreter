mod ast;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod semantic_analyzer;
mod symbols;
mod tokens;

use crate::interpreter::{Interpreter, NodeVisitor};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::semantic_analyzer::SemanticAnalyzer;
use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    let source = fs::read_to_string(args[1].clone())?;

    let lexer = Lexer::new(source.clone());
    let mut parser = Parser::new(lexer);
    let mut tree = parser.parse();

    let mut semantic_analyzer = SemanticAnalyzer::new();
    semantic_analyzer.visit(&mut tree);

    let mut interpreter = Interpreter::new();
    let _result = interpreter.visit(&mut tree);

    Ok(())
}
