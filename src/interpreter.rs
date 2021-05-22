use crate::ast::*;
use crate::parser::Parser;
use crate::tokens::{TokenType, Value};
use std::collections::HashMap;

pub trait NodeVisitor {
    fn visit_num(&mut self, num: &Num) -> Value;
    fn visit_bin_op(&mut self, bin_op: &BinOp) -> Value;
    fn visit_unary_op(&mut self, unary_op: &UnaryOp) -> Value;
    fn visit_compound(&mut self, compound: &Compound);
    fn visit_assign(&mut self, assign: &Assign);
    fn visit_var(&mut self, var: &Var) -> Value;
    fn visit_program(&mut self, program: &Program) -> Value;
    fn visit_block(&mut self, block: &Block);
    fn visit_var_decl(&mut self, var_decl: &VarDecl);
    fn visit_type(&mut self, type_: &Type);
    fn visit_procedure_decl(&mut self, procedure_decl: &ProcedureDecl);

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
            Node::VarDecl(n) => {
                self.visit_var_decl(n);
                Value::None
            }
            Node::ProcedureDecl(n) => {
                self.visit_procedure_decl(n);
                Value::None
            } // Node::Block(n) => {
              //     self.visit_block(n);
              //     Value::None
              // }
              // Node::Type(n) => {
              //     self.visit_type(n);
              //     Value::None
              // }
        }
    }
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

    pub fn print_memory(&self) {
        println!("GLOBAL_MEMORY contents: {:#?}", self.global_scope);
    }
}

impl NodeVisitor for Interpreter {
    fn visit_num(&mut self, num: &Num) -> Value {
        num.value.clone()
    }

    // TODO: This isn't quite right
    fn visit_bin_op(&mut self, bin_op: &BinOp) -> Value {
        let mut float = false;
        let left = match self.visit(&bin_op.left) {
            Value::Integer(l) => l as f32,
            Value::Float(l) => {
                float = true;
                l
            }
            _ => panic!(),
        };
        let right = match self.visit(&bin_op.right) {
            Value::Integer(r) => r as f32,
            Value::Float(r) => {
                float = true;
                r
            }
            _ => panic!(),
        };

        let res = match bin_op.op.type_ {
            TokenType::Plus => left + right,
            TokenType::Minus => left - right,
            TokenType::Mul => left * right,
            TokenType::IntegerDiv => (left as i32 / right as i32) as f32,
            TokenType::FloatDiv => left / right,
            _ => panic!(),
        };
        match float {
            true => Value::Float(res as f32),
            false => Value::Integer(res as i32),
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
                TokenType::Plus => Value::Integer(n),
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
        let var_name = assign.left.value.expect_string();
        let value = self.visit(&assign.right);
        self.global_scope.insert(var_name.to_lowercase(), value);
    }

    fn visit_var(&mut self, var: &Var) -> Value {
        let var_name = var.value.expect_string();
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
        for declaration in &block.declarations {
            self.visit(declaration);
        }
        self.visit(&block.compound_statement);
    }

    fn visit_var_decl(&mut self, _: &VarDecl) {}

    fn visit_type(&mut self, _: &Type) {}

    fn visit_procedure_decl(&mut self, _: &ProcedureDecl) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::tokens::{Token, TokenType, Value};

    #[test]
    fn binary_ops() {
        let mul = Token::new(TokenType::Mul, Value::Char('*'), 1, 1);
        let plus = Token::new(TokenType::Plus, Value::Char('+'), 1, 1);
        let two = Token::new(TokenType::Integer, Value::Float(2.0), 1, 1);
        let seven = Token::new(TokenType::Integer, Value::Float(7.0), 1, 1);
        let three = Token::new(TokenType::Integer, Value::Float(3.0), 1, 1);

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
        let five = Token::new(TokenType::Integer, Value::Float(5.0), 1, 1);
        let two = Token::new(TokenType::Integer, Value::Float(2.0), 1, 1);
        let minus = Token::new(TokenType::Minus, Value::Char('-'), 1, 1);

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
