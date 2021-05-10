use crate::tokens::{Token, TokenType, Value};

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

                '/' => {
                    self.advance();
                    return Token::new(TokenType::Div, Value::Char(c));
                }

                '(' => {
                    self.advance();
                    return Token::new(TokenType::LeftParen, Value::Char(c));
                }

                ')' => {
                    self.advance();
                    return Token::new(TokenType::RightParen, Value::Char(c));
                }

                _ => self.error(),
            }
        }
        Token::new(TokenType::EOF, Value::None)
    }
}
