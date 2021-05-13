use crate::lexer::Lexer;
use crate::tokens::{Token, TokenType, Value};

enum Node {
    BinOp(Box<BinOp>),
    Num(Num),
    UnaryOp(Box<UnaryOp>),
}

struct BinOp {
    left: Node,
    token: Token,
    op: Token,
    right: Node,
}

impl BinOp {
    fn new(left: Node, op: Token, right: Node) -> Self {
        BinOp {
            left,
            token: op.clone(),
            op,
            right,
        }
    }
}

struct Num {
    token: Token,
    value: Value,
}

impl Num {
    fn new(token: Token) -> Self {
        Num {
            value: token.value.clone(),
            token,
        }
    }
}

struct UnaryOp {
    token: Token,
    op: Token,
    expr: Node,
}

impl UnaryOp {
    fn new(op: Token, expr: Node) -> Self {
        UnaryOp {
            token: op.clone(),
            op,
            expr,
        }
    }
}

pub struct Parser {
    lexer: Lexer,
    current_token: Option<Token>,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = Some(lexer.get_next_token());
        Parser {
            lexer,
            current_token,
        }
    }

    fn error(&self) {
        panic!("Invalid syntax");
    }

    fn eat(&mut self, token_type: TokenType) {
        if self.current_token.as_ref().unwrap().type_ == token_type {
            self.current_token = Some(self.lexer.get_next_token());
        } else {
            self.error();
        }
    }

    fn factor(&mut self) -> Node {
        let token = self.current_token.clone().unwrap();
        match &token.type_ {
            TokenType::Plus => {
                self.eat(TokenType::Plus);
                Node::UnaryOp(Box::new(UnaryOp::new(token, self.factor())))
            }
            TokenType::Minus => {
                self.eat(TokenType::Minus);
                Node::UnaryOp(Box::new(UnaryOp::new(token, self.factor())))
            }
            TokenType::Integer => {
                self.eat(TokenType::Integer);
                Node::Num(Num::new(token))
            }
            TokenType::LeftParen => {
                self.eat(TokenType::LeftParen);
                let node = self.expr();
                self.eat(TokenType::RightParen);
                node
            }
            _ => unreachable!(),
        }
    }

    fn term(&mut self) -> Node {
        let mut node = self.factor();

        while let TokenType::Mul | TokenType::Div = self.current_token.as_ref().unwrap().type_ {
            let token = self.current_token.clone().unwrap();
            match token.type_ {
                TokenType::Mul => self.eat(TokenType::Mul),
                TokenType::Div => self.eat(TokenType::Div),
                _ => unimplemented!(),
            }
            node = Node::BinOp(Box::new(BinOp::new(node, token, self.factor())));
        }
        node
    }

    fn expr(&mut self) -> Node {
        let mut node = self.term();

        while let TokenType::Plus | TokenType::Minus = self.current_token.as_ref().unwrap().type_ {
            let token = self.current_token.clone().unwrap();
            match token.type_ {
                TokenType::Plus => self.eat(TokenType::Plus),
                TokenType::Minus => self.eat(TokenType::Minus),
                _ => unimplemented!(),
            }
            node = Node::BinOp(Box::new(BinOp::new(node, token, self.term())));
        }
        node
    }

    fn parse(&mut self) -> Node {
        self.expr()
    }
}

trait NodeVisitor {
    fn visit_num(&self, num: &Num) -> f32;
    fn visit_bin_op(&self, bin_op: &BinOp) -> f32;
    fn visit_unary_op(&self, unary_op: &UnaryOp) -> f32;
}

pub struct Interpreter {
    parser: Parser,
}

impl Interpreter {
    pub fn new(parser: Parser) -> Self {
        Interpreter { parser }
    }

    pub fn interpret(&mut self) -> f32 {
        let tree = self.parser.parse();
        self.visit(&tree)
    }

    fn visit(&self, node: &Node) -> f32 {
        match node {
            Node::BinOp(n) => self.visit_bin_op(n),
            Node::UnaryOp(n) => self.visit_unary_op(n),
            Node::Num(n) => self.visit_num(n),
        }
    }
}

impl NodeVisitor for Interpreter {
    fn visit_num(&self, num: &Num) -> f32 {
        match num.value {
            Value::Number(n) => n,
            _ => unimplemented!(),
        }
    }

    fn visit_bin_op(&self, bin_op: &BinOp) -> f32 {
        let left = self.visit(&bin_op.left);
        let right = self.visit(&bin_op.right);

        match bin_op.op.type_ {
            TokenType::Plus => left + right,
            TokenType::Minus => left - right,
            TokenType::Mul => left * right,
            TokenType::Div => left / right,
            _ => unimplemented!(),
        }
    }

    fn visit_unary_op(&self, unary_op: &UnaryOp) -> f32 {
        let expr = self.visit(&unary_op.expr);
        match unary_op.op.type_ {
            TokenType::Plus => (0 as f32) + expr,
            TokenType::Minus => (0 as f32) - expr,
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::tokens::{Token, TokenType, Value};

    #[test]
    fn binary_ops() {
        let mul = Token::new(TokenType::Mul, Value::Char('*'));
        let plus = Token::new(TokenType::Plus, Value::Char('+'));
        let two = Token::new(TokenType::Integer, Value::Number(2 as f32));
        let seven = Token::new(TokenType::Integer, Value::Number(7 as f32));
        let three = Token::new(TokenType::Integer, Value::Number(3 as f32));

        let add_node = Node::BinOp(Box::new(BinOp::new(
            Node::BinOp(Box::new(BinOp::new(
                Node::Num(Num::new(two)),
                mul,
                Node::Num(Num::new(seven)),
            ))),
            plus,
            Node::Num(Num::new(three)),
        )));

        // The string passed to lexer doesn't matter but it has to be valid syntax
        let lexer = Lexer::new("2 * 7 + 3".to_string());
        let parser = Parser::new(lexer);
        let inperpreter = Interpreter::new(parser);
        let res = inperpreter.visit(&add_node);
        assert_eq!(res, 17 as f32);
    }

    #[test]
    fn unary_op() {
        let five = Token::new(TokenType::Integer, Value::Number(5 as f32));
        let two = Token::new(TokenType::Integer, Value::Number(2 as f32));
        let minus = Token::new(TokenType::Minus, Value::Char('-'));

        // 5 - -2
        let expr_node = Node::BinOp(Box::new(BinOp::new(
            Node::Num(Num::new(five)),
            minus.clone(),
            Node::UnaryOp(Box::new(UnaryOp::new(
                minus.clone(),
                Node::UnaryOp(Box::new(UnaryOp::new(minus, Node::Num(Num::new(two))))),
            ))),
        )));

        // The string passed to lexer doesn't matter but it has to be valid syntax
        let lexer = Lexer::new("5 + -2".to_string());
        let parser = Parser::new(lexer);
        let inperpreter = Interpreter::new(parser);
        let res = inperpreter.visit(&expr_node);
        assert_eq!(res, 3 as f32);
    }
}
