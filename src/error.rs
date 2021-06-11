use crate::tokens::Token;
use std::fmt::{self, Display, Formatter};

pub enum ErrorCode {
    UnexpectedToken,
    IDNotFound,
    DuplicateID,
    WrongParamsNum,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::DuplicateID => write!(f, "Duplicate id found"),
            ErrorCode::UnexpectedToken => write!(f, "Unexpected token"),
            ErrorCode::IDNotFound => write!(f, "Identifier not found"),
            ErrorCode::WrongParamsNum => write!(f, "Wrong number of params"),
        }
    }
}

pub struct LexerError {
    message: String,
}

impl LexerError {
    pub fn new(message: String) -> Self {
        LexerError { message }
    }

    pub fn throw(self) {
        panic!("Lexer Error: {}", self.message)
    }
}

pub struct ParserError {
    message: String,
    // error_code: ErrorCode,
    // token: Token,
}

impl ParserError {
    pub fn new(message: String, _error_code: ErrorCode, _token: Token) -> Self {
        ParserError {
            message,
            // error_code,
            // token,
        }
    }

    pub fn throw(self) {
        panic!("Parser Error: {}", self.message)
    }
}

pub struct SemanticError {
    message: String,
    // error_code: ErrorCode,
    // token: Token,
}

impl SemanticError {
    pub fn new(message: String, // , _error_code: ErrorCode, _token: Token
    ) -> Self {
        SemanticError {
            message,
            // error_code,
            // token,
        }
    }

    pub fn throw(self) {
        panic!("Semantic Error: {}", self.message)
    }
}
