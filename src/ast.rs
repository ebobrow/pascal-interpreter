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
    Program(Box<Program>),
    Block(Box<Block>),
    VarDecl(VarDecl),
    Type(Type),
    NoOp,
}

struct Program {
    name: String,
    block: Block,
}

impl Program {
    fn new(name: String, block: Block) -> Self {
        Program { name, block }
    }
}

struct Block {
    declarations: Vec<VarDecl>,
    compound_statement: Node,
}

impl Block {
    fn new(declarations: Vec<VarDecl>, compound_statement: Node) -> Self {
        Block {
            declarations,
            compound_statement,
        }
    }
}

#[derive(Clone)]
struct VarDecl {
    var_node: Var,
    type_node: Type,
}

impl VarDecl {
    fn new(var_node: Var, type_node: Type) -> Self {
        VarDecl {
            var_node,
            type_node,
        }
    }
}

#[derive(Clone)]
struct Type {
    token: Token,
    value: Value,
}

impl Type {
    fn new(token: Token) -> Self {
        Type {
            value: token.value.clone(),
            token,
        }
    }
}

// TODO: token field seems unnecessary?
#[derive(Clone)]
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
            // self.error();
            panic!("Unexpected token");
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
            TokenType::IntegerConst => {
                self.eat(TokenType::IntegerConst);
                Node::Num(Num::new(token))
            }
            TokenType::RealConst => {
                self.eat(TokenType::RealConst);
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

        while let TokenType::Mul | TokenType::IntegerDiv | TokenType::FloatDiv =
            self.current_token.as_ref().unwrap().type_
        {
            let token = self.current_token.clone().unwrap();
            self.eat(token.clone().type_);
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
            // self.error();
            panic!("Unexpected ID");
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
        self.eat(TokenType::Program);
        let var_node = self.variable();
        let prog_name = match var_node.value {
            Value::String(s) => s,
            _ => panic!("Error"),
        };
        self.eat(TokenType::Semi);
        let block_node = self.block();
        let program_node = Program::new(prog_name, block_node);
        self.eat(TokenType::Dot);

        Node::Program(Box::new(program_node))
    }

    fn block(&mut self) -> Block {
        let declaration_nodes = self.declarations();
        let compound_statement_node = self.compound_statement();
        Block::new(declaration_nodes, compound_statement_node)
    }

    fn declarations(&mut self) -> Vec<VarDecl> {
        let mut declarations = Vec::new();
        if let TokenType::Var = self.current_token.as_ref().unwrap().type_ {
            self.eat(TokenType::Var);
            while let TokenType::ID = self.current_token.as_ref().unwrap().type_ {
                declarations.append(&mut self.variable_declaration());
                self.eat(TokenType::Semi);
            }
        }
        declarations
    }

    fn variable_declaration(&mut self) -> Vec<VarDecl> {
        let mut var_nodes = vec![Var::new(self.current_token.clone().unwrap())];
        self.eat(TokenType::ID);

        while let TokenType::Comma = self.current_token.as_ref().unwrap().type_ {
            self.eat(TokenType::Comma);
            var_nodes.push(Var::new(self.current_token.clone().unwrap()));
            self.eat(TokenType::ID);
        }

        self.eat(TokenType::Colon);

        let type_node = self.type_spec();
        let mut var_declarations = Vec::new();
        for node in var_nodes {
            var_declarations.push(VarDecl::new(node, type_node.clone()));
        }
        var_declarations
    }

    fn type_spec(&mut self) -> Type {
        let token = self.current_token.clone().unwrap();
        if let TokenType::Integer = token.type_ {
            self.eat(TokenType::Integer);
        } else {
            self.eat(TokenType::Real);
        }

        Type::new(token)
    }

    fn parse(&mut self) -> Node {
        let node = self.program();
        if let TokenType::EOF = self.current_token.as_ref().unwrap().type_ {
            return node;
        } else {
            // self.error();
            panic!("Stuff after the end");
        }
        unreachable!()
    }
}

trait NodeVisitor {
    fn visit_num(&self, num: &Num) -> Value;
    fn visit_bin_op(&mut self, bin_op: &BinOp) -> Value;
    fn visit_unary_op(&mut self, unary_op: &UnaryOp) -> Value;
    fn visit_compound(&mut self, compound: &Compound);
    fn visit_assign(&mut self, assign: &Assign);
    fn visit_var(&self, var: &Var) -> Value;
    fn visit_program(&mut self, program: &Program) -> Value;
    fn visit_block(&mut self, block: &Block);
    fn visit_var_decl(&self, var_decl: &VarDecl);
    fn visit_type(&self, type_: &Type);
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
            Node::BinOp(n) => self.visit_bin_op(n),
            Node::UnaryOp(n) => self.visit_unary_op(n),
            Node::Num(n) => self.visit_num(n),
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
            Node::Program(n) => self.visit_program(n),
            Node::Block(n) => {
                self.visit_block(n);
                Value::None
            }
            Node::VarDecl(n) => {
                self.visit_var_decl(n);
                Value::None
            }
            Node::Type(n) => {
                self.visit_type(n);
                Value::None
            }
        }
    }
}

impl NodeVisitor for Interpreter {
    fn visit_num(&self, num: &Num) -> Value {
        num.value.clone()
    }

    fn visit_bin_op(&mut self, bin_op: &BinOp) -> Value {
        let left = self.visit(&bin_op.left);
        let right = self.visit(&bin_op.right);

        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(match bin_op.op.type_ {
                TokenType::Plus => l + r,
                TokenType::Minus => l - r,
                TokenType::Mul => l * r,
                TokenType::IntegerDiv => l / r,
                _ => panic!("Error"),
            }),
            (Value::Float(l), Value::Float(r)) => Value::Float(match bin_op.op.type_ {
                TokenType::Plus => l + r,
                TokenType::Minus => l - r,
                TokenType::Mul => l * r,
                TokenType::FloatDiv => l / r,
                _ => panic!("Error"),
            }),
            _ => panic!("Error"),
        }
    }

    fn visit_unary_op(&mut self, unary_op: &UnaryOp) -> Value {
        match self.visit(&unary_op.expr) {
            Value::Float(n) => match unary_op.op.type_ {
                TokenType::Plus => Value::Float((0.0) + n),
                TokenType::Minus => Value::Float((0.0) - n),
                _ => unimplemented!(),
            },
            Value::Integer(n) => match unary_op.op.type_ {
                TokenType::Plus => Value::Integer(0 + n),
                TokenType::Minus => Value::Integer(0 - n),
                _ => unimplemented!(),
            },
            _ => panic!("Error"),
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

    fn visit_program(&mut self, program: &Program) -> Value {
        self.visit_block(&program.block);
        Value::None
    }

    fn visit_block(&mut self, block: &Block) {
        for declaration in block.declarations.clone() {
            self.visit(&Node::VarDecl(declaration));
        }
        self.visit(&block.compound_statement);
    }

    fn visit_var_decl(&self, _: &VarDecl) {
        todo!()
    }

    fn visit_type(&self, _: &Type) {
        todo!()
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
        let two = Token::new(TokenType::Integer, Value::Float(2.0));
        let seven = Token::new(TokenType::Integer, Value::Float(7.0));
        let three = Token::new(TokenType::Integer, Value::Float(3.0));

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
        assert_eq!(res, Value::Float(17.0));
    }

    #[test]
    fn unary_op() {
        let five = Token::new(TokenType::Integer, Value::Float(5.0));
        let two = Token::new(TokenType::Integer, Value::Float(2.0));
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
        assert_eq!(res, Value::Float(3.0));
    }

    #[test]
    fn variables() {
        let text = "
PROGRAM VarTest;

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
        expected.insert(String::from("number"), Value::Integer(2));
        expected.insert(String::from("a"), Value::Integer(2));
        expected.insert(String::from("_x"), Value::Integer(11));
        expected.insert(String::from("c"), Value::Integer(27));
        expected.insert(String::from("b"), Value::Integer(25));

        assert_eq!(interpreter.global_scope, expected);
    }
}
