use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // Types
    Integer,
    Real,
    IntegerConst,
    RealConst,

    // Operators
    Plus,
    Minus,
    Mul,
    IntegerDiv,
    FloatDiv,
    RightParen,
    LeftParen,

    // Reserved keywords
    Program,
    Var,
    Begin,
    End,
    Procedure,

    ID,
    Assign,
    Semi,
    Dot,
    Colon,
    Comma,

    EOF,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Float(f32),
    Integer(i32),
    Char(char),
    String(String),
    None,
}

impl Value {
    pub fn expect_string(&self) -> String {
        match self {
            Value::String(s) => s.to_string(),
            _ => panic!("Expected string"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Float(v) => write!(f, "{}", v),
            Value::Integer(v) => write!(f, "{}", v),
            Value::Char(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::None => write!(f, ""),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
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
