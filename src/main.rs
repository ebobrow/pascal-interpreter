use std::fmt::{self, Display, Formatter};
use std::io::{stdin, stdout, Write};

#[derive(Debug, PartialEq, Clone)]
enum TokenType {
    Integer,
    Plus,
    Minus,
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
    text: String,
    pos: usize,
    current_token: Option<Token>,
    current_char: Option<char>,
}

impl Interpreter {
    fn new(text: String) -> Self {
        Interpreter {
            text: text.clone(),
            pos: 0,
            current_token: None,
            current_char: Some(text.as_bytes()[0] as char),
        }
    }

    fn error(&self) {
        panic!("Error parsing input");
    }

    fn advance(&mut self) {
        self.pos += 1;
        if self.pos > self.text.len() - 1 {
            self.current_char = None;
        } else {
            self.current_char = Some(self.text.as_bytes()[self.pos] as char)
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn integer(&mut self) -> f32 {
        let mut result = String::new();
        while let Some(n) = self.current_char {
            if n.is_numeric() {
                result.push(n);
                self.advance();
            } else {
                break;
            }
        }
        result.parse().unwrap()
    }

    fn get_next_token(&mut self) -> Token {
        while let Some(c) = self.current_char {
            if c.is_whitespace() {
                self.skip_whitespace();
                continue;
            }

            if c.is_numeric() {
                return Token::new(TokenType::Integer, Value::Number(self.integer()));
            }

            if c == '+' {
                self.advance();
                return Token::new(TokenType::Plus, Value::Char(c));
            }

            if c == '-' {
                self.advance();
                return Token::new(TokenType::Minus, Value::Char(c));
            }

            self.error();
        }
        Token::new(TokenType::EOF, Value::None)
    }

    fn eat(&mut self, type_: TokenType) {
        if self.current_token.as_ref().unwrap().type_ == type_ {
            self.current_token = Some(self.get_next_token());
        } else {
            self.error()
        }
    }

    fn term(&mut self) -> Value {
        let token = self.current_token.clone();
        self.eat(TokenType::Integer);
        token.unwrap().value
    }

    fn expr(&mut self) -> f32 {
        self.current_token = Some(self.get_next_token());

        let mut result = match self.term() {
            Value::Number(n) => n,
            _ => {
                self.error();
                unreachable!()
            }
        };
        while let TokenType::Plus | TokenType::Minus = self.current_token.as_ref().unwrap().type_ {
            let token = self.current_token.as_ref().unwrap().clone();
            match token.type_ {
                TokenType::Plus => {
                    self.eat(TokenType::Plus);
                    result += match self.term() {
                        Value::Number(n) => n,
                        _ => {
                            self.error();
                            unreachable!()
                        }
                    }
                }
                TokenType::Minus => {
                    self.eat(TokenType::Minus);
                    result -= match self.term() {
                        Value::Number(n) => n,
                        _ => {
                            self.error();
                            unreachable!()
                        }
                    }
                }
                _ => unimplemented!(),
            }
        }
        result
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
