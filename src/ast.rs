use crate::lexer::Lexer;
use crate::tokens::{Token, TokenType, Value};

enum Node {
    BinOp(Box<BinOp>),
    Num(Box<Num>),
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
        let type_ = &token.type_;
        if let TokenType::Integer = type_ {
            self.eat(TokenType::Integer);
            return Node::Num(Box::new(Num::new(token)));
        } else if let TokenType::LeftParen = type_ {
            self.eat(TokenType::LeftParen);
            let node = self.expr();
            self.eat(TokenType::RightParen);
            return node;
        } else {
            unreachable!()
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
        match &tree {
            Node::BinOp(b) => self.visit_bin_op(b),
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
        let left = match &bin_op.left {
            Node::BinOp(b) => self.visit_bin_op(b),
            Node::Num(n) => self.visit_num(n),
        };
        let right = match &bin_op.right {
            Node::BinOp(b) => self.visit_bin_op(b),
            Node::Num(n) => self.visit_num(n),
        };

        match bin_op.op.type_ {
            TokenType::Plus => left + right,
            TokenType::Minus => left - right,
            TokenType::Mul => left * right,
            TokenType::Div => left / right,
            _ => unimplemented!(),
        }
    }
}
