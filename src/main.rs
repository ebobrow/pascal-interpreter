use std::fmt::{self, Display, Formatter};
use std::io::{stdin, stdout, Write};

#[derive(Debug, PartialEq, Clone)]
enum TokenType {
    Integer,
    Plus,
    EOF,
}

#[derive(Clone)]
enum Value {
    Number(f32),
    Char(char),
    None,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Char(c) => write!(f, "{}", c),
            Value::None => write!(f, ""),
        }
    }
}

#[derive(Clone)]
struct Token {
    type_: TokenType,
    value: Value,
}

impl Token {
    fn new(type_: TokenType, value: Value) -> Self {
        Token { type_, value }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Token({:?}, {})", self.type_, self.value.to_string())
    }
}

struct Interpreter {
    text: String, // ?
    pos: usize,
    current_token: Option<Token>,
}

impl Interpreter {
    fn new(text: String) -> Self {
        Interpreter {
            text,
            pos: 0,
            current_token: None,
        }
    }

    fn error(&self) {
        panic!("Error parsing input");
    }

    fn get_next_token(&mut self) -> Token {
        if self.pos > self.text.len() - 1 {
            return Token::new(TokenType::EOF, Value::None);
        }

        let current_char = self.text.as_bytes()[self.pos] as char;
        if current_char.is_numeric() {
            self.pos += 1;
            return Token::new(
                TokenType::Integer,
                Value::Number(current_char.to_digit(10).unwrap() as f32),
            );
        }

        if current_char == '+' {
            self.pos += 1;
            return Token::new(TokenType::Plus, Value::Char(current_char));
        }

        self.error();
        unreachable!()
    }

    fn eat(&mut self, type_: TokenType) {
        if self.current_token.as_ref().unwrap().type_ == type_ {
            self.current_token = Some(self.get_next_token());
        } else {
            self.error()
        }
    }

    fn expr(&mut self) -> f32 {
        self.current_token = Some(self.get_next_token());

        let left = &self.current_token.as_ref().unwrap().clone();
        self.eat(TokenType::Integer);

        let op = &self.current_token.as_ref().unwrap().clone();
        self.eat(TokenType::Plus);

        let right = &self.current_token.as_ref().unwrap().clone();
        self.eat(TokenType::Integer);

        match (&left.value, &right.value) {
            (Value::Number(l), Value::Number(r)) => l + r,
            _ => {
                self.error();
                unreachable!()
            }
        }
    }
}

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
            let mut interpreter = Interpreter::new(input);
            let result = interpreter.expr();
            println!("{}", result);
        }
    }
}
