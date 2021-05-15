use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Integer,
    Plus,
    Minus,
    Mul,
    Div,
    RightParen,
    LeftParen,
    Begin,
    End,
    ID,
    Assign,
    Semi,
    Dot,
    EOF,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Number(f32),
    Char(char),
    String(String),
    None,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Char(c) => write!(f, "{}", c),
            Value::String(s) => write!(f, "{}", s),
            Value::None => write!(f, ""),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Token {
    pub type_: TokenType,
    pub value: Value,
}

impl Token {
    pub fn new(type_: TokenType, value: Value) -> Self {
        Token { type_, value }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Token({:?}, {})", self.type_, self.value.to_string())
    }
}
