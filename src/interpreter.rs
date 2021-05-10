use crate::lexer::Lexer;
use crate::tokens::{Token, TokenType, Value};

pub struct Interpreter {
    lexer: Lexer,
    current_token: Option<Token>,
}

impl Interpreter {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = Some(lexer.get_next_token());
        Interpreter {
            lexer,
            current_token,
        }
    }

    fn error(&self) {
        panic!("Invalid syntax");
    }

    fn eat(&mut self, type_: TokenType) {
        if self.current_token.as_ref().unwrap().type_ == type_ {
            self.current_token = Some(self.lexer.get_next_token());
        } else {
            self.error()
        }
    }

    fn factor(&mut self) -> f32 {
        let token = self.current_token.clone();
        let type_ = &token.as_ref().unwrap().type_;
        if let TokenType::Integer = type_ {
            self.eat(TokenType::Integer);
            return match token.unwrap().value {
                Value::Number(n) => n,
                _ => {
                    self.error();
                    unreachable!()
                }
            };
        } else if let TokenType::LeftParen = type_ {
            self.eat(TokenType::LeftParen);
            let result = self.expr();
            self.eat(TokenType::RightParen);
            return result;
        } else {
            self.error();
            unreachable!()
        }
    }

    fn term(&mut self) -> f32 {
        let mut result = self.factor();

        while let TokenType::Mul | TokenType::Div = self.current_token.as_ref().unwrap().type_ {
            let token = self.current_token.clone().unwrap();

            match token.type_ {
                TokenType::Mul => {
                    self.eat(TokenType::Mul);
                    result *= self.factor();
                }
                TokenType::Div => {
                    self.eat(TokenType::Div);
                    result /= self.factor();
                }
                _ => unimplemented!(),
            }
        }
        result
    }

    pub fn expr(&mut self) -> f32 {
        let mut result = self.term();

        while let TokenType::Plus | TokenType::Minus = self.current_token.as_ref().unwrap().type_ {
            let token = self.current_token.clone().unwrap();

            match token.type_ {
                TokenType::Plus => {
                    self.eat(TokenType::Plus);
                    result += self.term();
                }
                TokenType::Minus => {
                    self.eat(TokenType::Minus);
                    result -= self.term();
                }
                _ => unimplemented!(),
            }
        }
        result
    }
}
