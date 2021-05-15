use crate::tokens::{Token, TokenType, Value};
use phf::phf_map;

const RESERVED_KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "BEGIN" => TokenType::Begin,
    "END" => TokenType::End,
    "DIV" => TokenType::Div
};

pub struct Lexer {
    text: String,
    pos: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(text: String) -> Self {
        Lexer {
            text: text.clone(),
            pos: 0,
            current_char: Some(text.as_bytes()[0] as char),
        }
    }

    fn error(&self) {
        panic!("Invalid character");
    }

    fn advance(&mut self) {
        self.pos += 1;
        if self.pos > self.text.len() - 1 {
            self.current_char = None;
        } else {
            self.current_char = Some(self.text.as_bytes()[self.pos] as char);
        }
    }

    fn skip_whitespace(&mut self) {
        while self.current_char.filter(|c| c.is_whitespace()).is_some() {
            self.advance();
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

    pub fn get_next_token(&mut self) -> Token {
        while let Some(c) = self.current_char {
            if c.is_whitespace() {
                self.skip_whitespace();
                continue;
            }

            if c.is_numeric() {
                return Token::new(TokenType::Integer, Value::Number(self.integer()));
            }

            match c {
                '+' => {
                    self.advance();
                    return Token::new(TokenType::Plus, Value::Char(c));
                }

                '-' => {
                    self.advance();
                    return Token::new(TokenType::Minus, Value::Char(c));
                }

                '*' => {
                    self.advance();
                    return Token::new(TokenType::Mul, Value::Char(c));
                }

                // '/' => {
                //     self.advance();
                //     return Token::new(TokenType::Div, Value::Char(c));
                // }
                '(' => {
                    self.advance();
                    return Token::new(TokenType::LeftParen, Value::Char(c));
                }

                ')' => {
                    self.advance();
                    return Token::new(TokenType::RightParen, Value::Char(c));
                }

                ':' => {
                    if let Some('=') = self.peek() {
                        self.advance();
                        self.advance();
                        return Token::new(TokenType::Assign, Value::String(String::from(":=")));
                    }
                }

                ';' => {
                    self.advance();
                    return Token::new(TokenType::Semi, Value::Char(c));
                }

                '.' => {
                    self.advance();
                    return Token::new(TokenType::Dot, Value::Char(c));
                }

                c => {
                    if c.is_alphabetic() || c == '_' {
                        return self.id();
                    } else {
                        self.error()
                    }
                }
            }
        }
        Token::new(TokenType::EOF, Value::None)
    }

    fn peek(&self) -> Option<char> {
        if self.pos > self.text.len() {
            None
        } else {
            Some(self.text.as_bytes()[self.pos + 1] as char)
        }
    }

    fn id(&mut self) -> Token {
        let mut result = String::new();
        while let Some(c) = self
            .current_char
            .filter(|c| c.is_alphanumeric() || c == &'_')
        {
            result.push(c);
            self.advance();
        }

        RESERVED_KEYWORDS.get(&result[..]).map_or(
            Token::new(TokenType::ID, Value::String(result.clone())),
            |t| Token::new(t.clone(), Value::String(result)),
        )
    }
}
