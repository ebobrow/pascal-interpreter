use crate::lexer::Lexer;
use crate::tokens::{Token, TokenType, Value};
use std::collections::HashMap;

// TODO: Instead of structs for node types, store data directly in enum
enum Node {
    BinOp(Box<BinOp>),
    Num(Num),
    UnaryOp(Box<UnaryOp>),
    Compound(Compound),
    Assign(Box<Assign>),
    Var(Var),
    NoOp,
}

// TODO: token field seems unnecessary?
struct Var {
    token: Token,
    value: Value,
}

impl Var {
    fn new(token: Token) -> Self {
        Var {
            value: token.value.clone(),
            token,
        }
    }
}

struct Assign {
    left: Var,
    token: Token,
    op: Token,
    right: Node,
}

impl Assign {
    fn new(left: Var, op: Token, right: Node) -> Self {
        Assign {
            left,
            token: op.clone(),
            op,
            right,
        }
    }
}

struct Compound {
    children: Vec<Node>,
}

impl Compound {
    fn new() -> Self {
        Compound {
            children: Vec::new(),
        }
    }
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
            _ => Node::Var(self.variable()),
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

    fn empty(&self) -> Node {
        Node::NoOp
    }

    fn variable(&mut self) -> Var {
        let node = Var::new(self.current_token.clone().unwrap());
        self.eat(TokenType::ID);
        node
    }

    fn assignment_statement(&mut self) -> Node {
        let left = self.variable();
        let token = self.current_token.clone().unwrap();
        self.eat(TokenType::Assign);
        let right = self.expr();
        Node::Assign(Box::new(Assign::new(left, token, right)))
    }

    fn statement(&mut self) -> Node {
        match self.current_token.as_ref().unwrap().type_ {
            TokenType::Begin => self.compound_statement(),
            TokenType::ID => self.assignment_statement(),
            _ => self.empty(),
        }
    }

    fn statement_list(&mut self) -> Vec<Node> {
        let node = self.statement();

        let mut results = vec![node];

        while let TokenType::Semi = self.current_token.as_ref().unwrap().type_ {
            self.eat(TokenType::Semi);
            results.push(self.statement());
        }

        if let TokenType::ID = self.current_token.as_ref().unwrap().type_ {
            self.error();
        }

        results
    }

    fn compound_statement(&mut self) -> Node {
        self.eat(TokenType::Begin);
        let nodes = self.statement_list();
        self.eat(TokenType::End);

        let mut root = Compound::new();
        for node in nodes {
            root.children.push(node);
        }
        Node::Compound(root)
    }

    fn program(&mut self) -> Node {
        let node = self.compound_statement();
        self.eat(TokenType::Dot);
        node
    }

    fn parse(&mut self) -> Node {
        let node = self.program();
        if let TokenType::EOF = self.current_token.as_ref().unwrap().type_ {
            return node;
        } else {
            self.error();
        }
        unreachable!()
    }
}

trait NodeVisitor {
    fn visit_num(&self, num: &Num) -> f32;
    fn visit_bin_op(&mut self, bin_op: &BinOp) -> f32;
    fn visit_unary_op(&mut self, unary_op: &UnaryOp) -> f32;
    fn visit_compound(&mut self, compound: &Compound);
    fn visit_assign(&mut self, assign: &Assign);
    fn visit_var(&self, var: &Var) -> Value;
}

pub struct Interpreter {
    parser: Parser,
    global_scope: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new(parser: Parser) -> Self {
        Interpreter {
            parser,
            global_scope: HashMap::new(),
        }
    }

    pub fn interpret(&mut self) -> Value {
        let tree = self.parser.parse();
        self.visit(&tree)
    }

    fn visit(&mut self, node: &Node) -> Value {
        match node {
            Node::BinOp(n) => Value::Number(self.visit_bin_op(n)),
            Node::UnaryOp(n) => Value::Number(self.visit_unary_op(n)),
            Node::Num(n) => Value::Number(self.visit_num(n)),
            Node::Compound(n) => {
                self.visit_compound(n);
                Value::None
            }
            Node::NoOp => Value::None,
            Node::Assign(n) => {
                self.visit_assign(n);
                Value::None
            }
            Node::Var(n) => self.visit_var(n),
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

    fn visit_bin_op(&mut self, bin_op: &BinOp) -> f32 {
        let left = match self.visit(&bin_op.left) {
            Value::Number(n) => n,
            _ => panic!("Error"),
        };
        let right = match self.visit(&bin_op.right) {
            Value::Number(n) => n,
            _ => panic!("Error"),
        };

        match bin_op.op.type_ {
            TokenType::Plus => left + right,
            TokenType::Minus => left - right,
            TokenType::Mul => left * right,
            TokenType::Div => left / right,
            _ => unimplemented!(),
        }
    }

    fn visit_unary_op(&mut self, unary_op: &UnaryOp) -> f32 {
        let expr = match self.visit(&unary_op.expr) {
            Value::Number(n) => n,
            _ => panic!("Error"),
        };
        match unary_op.op.type_ {
            TokenType::Plus => (0.0) + expr,
            TokenType::Minus => (0.0) - expr,
            _ => unimplemented!(),
        }
    }

    fn visit_compound(&mut self, compound: &Compound) {
        for child in &compound.children {
            self.visit(child);
        }
    }

    fn visit_assign(&mut self, assign: &Assign) {
        let var_name = match &assign.left.value {
            Value::String(s) => s.to_string(),
            _ => {
                panic!("Error");
            }
        };
        let value = self.visit(&assign.right);
        self.global_scope.insert(var_name.to_lowercase(), value);
    }

    fn visit_var(&self, var: &Var) -> Value {
        let var_name = match &var.value {
            Value::String(s) => s.to_string(),
            _ => panic!("Error"),
        };
        self.global_scope
            .get(&var_name.to_lowercase())
            .unwrap()
            .clone()
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
        let two = Token::new(TokenType::Integer, Value::Number(2.0));
        let seven = Token::new(TokenType::Integer, Value::Number(7.0));
        let three = Token::new(TokenType::Integer, Value::Number(3.0));

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
        let mut inperpreter = Interpreter::new(parser);
        let res = inperpreter.visit(&add_node);
        assert_eq!(res, Value::Number(17.0));
    }

    #[test]
    fn unary_op() {
        let five = Token::new(TokenType::Integer, Value::Number(5.0));
        let two = Token::new(TokenType::Integer, Value::Number(2.0));
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
        let mut inperpreter = Interpreter::new(parser);
        let res = inperpreter.visit(&expr_node);
        assert_eq!(res, Value::Number(3.0));
    }

    #[test]
    fn variables() {
        let text = "
BEGIN
    BEGIN
        number := 2;
        a := nUmbEr;
        b := 10 * a + 10 * NUMBER DIV 4;
        c := a - - b;
    END;

    _x := 11;
END.";

        let lexer = Lexer::new(text.to_string());
        let parser = Parser::new(lexer);
        let mut interpreter = Interpreter::new(parser);
        interpreter.interpret();

        let mut expected: HashMap<String, Value> = HashMap::new();
        expected.insert(String::from("number"), Value::Number(2.0));
        expected.insert(String::from("a"), Value::Number(2.0));
        expected.insert(String::from("_x"), Value::Number(11.0));
        expected.insert(String::from("c"), Value::Number(27.0));
        expected.insert(String::from("b"), Value::Number(25.0));

        assert_eq!(interpreter.global_scope, expected);
    }
}
