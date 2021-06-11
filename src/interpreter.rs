use crate::ast::*;
// use crate::parser::Parser;
use crate::symbols::{ARType, ActivationRecord, CallStack};
use crate::tokens::{TokenType, Value};

pub trait NodeVisitor {
    fn visit_num(&mut self, num: &mut Node) -> Value;
    fn visit_bin_op(&mut self, bin_op: &mut Node) -> Value;
    fn visit_unary_op(&mut self, unary_op: &mut Node) -> Value;
    fn visit_compound(&mut self, compound: &mut Compound) -> Value;
    fn visit_assign(&mut self, assign: &mut Node) -> Value;
    fn visit_var(&mut self, var: &mut Var) -> Value;
    fn visit_program(&mut self, program: &mut Node) -> Value;
    fn visit_block(&mut self, block: &mut Block) -> Value;
    fn visit_var_decl(&mut self, var_decl: &mut Node) -> Value;
    fn visit_type(&mut self, type_: &mut Type) -> Value;
    fn visit_procedure_decl(&mut self, procedure_decl: &mut Node) -> Value;
    fn visit_procedure_call(&mut self, procedure_call: &mut ProcedureCall) -> Value;

    fn visit(&mut self, node: &mut Node) -> Value {
        match node {
            Node::BinOp(..) => self.visit_bin_op(node),
            Node::UnaryOp(..) => self.visit_unary_op(node),
            Node::Num(..) => self.visit_num(node),
            Node::Compound(n) => self.visit_compound(n),
            Node::Assign(..) => self.visit_assign(node),
            Node::Var(n) => self.visit_var(n),
            Node::Program(..) => self.visit_program(node),
            Node::VarDecl(..) => self.visit_var_decl(node),
            Node::ProcedureDecl(..) => self.visit_procedure_decl(node),
            Node::ProcedureCall(n) => self.visit_procedure_call(n),
            // Node::Block(n) => self.visit_block(n),
            Node::NoOp => Value::None,
        }
    }
}

pub struct Interpreter {
    call_stack: CallStack,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            call_stack: CallStack::new(),
        }
    }
}

impl NodeVisitor for Interpreter {
    fn visit_num(&mut self, num: &mut Node) -> Value {
        if let Node::Num(value) = num {
            value.clone()
        } else {
            unreachable!()
        }
    }

    fn visit_bin_op(&mut self, bin_op: &mut Node) -> Value {
        if let Node::BinOp(left, op, right) = bin_op {
            let mut float = false;
            let left = match self.visit(left) {
                Value::Integer(l) => l as f32,
                Value::Float(l) => {
                    float = true;
                    l
                }
                _ => panic!(),
            };
            let right = match self.visit(right) {
                Value::Integer(r) => r as f32,
                Value::Float(r) => {
                    float = true;
                    r
                }
                _ => panic!(),
            };

            let res = match op.type_ {
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
        } else {
            unreachable!()
        }
    }

    fn visit_unary_op(&mut self, unary_op: &mut Node) -> Value {
        if let Node::UnaryOp(op, expr) = unary_op {
            match self.visit(expr) {
                Value::Float(n) => match op.type_ {
                    TokenType::Plus => Value::Float((0.0) + n),
                    TokenType::Minus => Value::Float((0.0) - n),
                    _ => unimplemented!(),
                },
                Value::Integer(n) => match op.type_ {
                    TokenType::Plus => Value::Integer(n),
                    TokenType::Minus => Value::Integer(0 - n),
                    _ => unimplemented!(),
                },
                _ => panic!("Error"),
            }
        } else {
            unreachable!()
        }
    }

    fn visit_compound(&mut self, compound: &mut Compound) -> Value {
        for child in &mut compound.children {
            self.visit(child);
        }

        Value::None
    }

    fn visit_assign(&mut self, assign: &mut Node) -> Value {
        if let Node::Assign(left, _, right) = assign {
            let value = self.visit(right);
            let ar = self.call_stack.peek().unwrap();
            ar.set(left.value.expect_string(), value);
        }

        Value::None
    }

    fn visit_var(&mut self, var: &mut Var) -> Value {
        let ar = self.call_stack.peek().unwrap();
        ar.get(var.value.expect_string().to_lowercase())
            .unwrap()
            .clone()
    }

    fn visit_program(&mut self, program: &mut Node) -> Value {
        if let Node::Program(name, block) = program {
            println!("ENTER PROGRAM: {}", &name);
            self.call_stack
                .push(ActivationRecord::new(name.clone(), ARType::Program, 1));
            self.visit_block(block);
            println!("{:#?}", self.call_stack);
            println!("EXIT PROGRAM: {}", &name);
            if let Some(ar) = self.call_stack.peek() {
                // Keep outermost ar for tests
                if ar.nesting_level != 1 {
                    self.call_stack.pop();
                }
            }
        }
        Value::None
    }

    fn visit_block(&mut self, block: &mut Block) -> Value {
        for declaration in &mut block.declarations {
            self.visit(declaration);
        }
        self.visit(&mut block.compound_statement);

        Value::None
    }

    fn visit_var_decl(&mut self, _: &mut Node) -> Value {
        Value::None
    }

    fn visit_type(&mut self, _: &mut Type) -> Value {
        Value::None
    }

    fn visit_procedure_decl(&mut self, _: &mut Node) -> Value {
        Value::None
    }

    fn visit_procedure_call(&mut self, procedure_call: &mut ProcedureCall) -> Value {
        let mut ar = ActivationRecord::new(procedure_call.proc_name.clone(), ARType::Procedure, 2);

        let formal_params = &procedure_call
            .proc_symbol
            .as_mut()
            .unwrap()
            .formal_params
            .clone();
        let actual_params = &mut procedure_call.actual_params;
        for (param_symbol, argument_node) in formal_params.iter().zip(actual_params.iter_mut()) {
            ar.set(param_symbol.name.clone(), self.visit(argument_node));
        }

        self.call_stack.push(ar);
        println!("ENTER PROCEDURE: {}", &procedure_call.proc_name);

        self.visit_block(
            &mut (procedure_call
                .proc_symbol
                .as_ref()
                .unwrap()
                .block_ast
                .as_ref()
                .unwrap()
                .clone()),
        );

        println!("{:#?}", self.call_stack);
        println!("EXIT PROCEDURE: {}", &procedure_call.proc_name);
        self.call_stack.pop();

        Value::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        lexer::Lexer,
        parser::Parser,
        tokens::{Token, TokenType, Value},
    };

    #[test]
    fn binary_ops() {
        let mul = Token::new(TokenType::Mul, Value::Char('*'), 1, 1);
        let plus = Token::new(TokenType::Plus, Value::Char('+'), 1, 1);
        let two = Value::Float(2.0);
        let seven = Value::Float(7.0);
        let three = Value::Float(3.0);

        let mut add_node = Node::BinOp(
            // left
            Box::new(Node::BinOp(
                // left
                Box::new(Node::Num(two)),
                // op
                mul,
                // right
                Box::new(Node::Num(seven)),
            )),
            // op
            plus,
            // right
            Box::new(Node::Num(three)),
        );

        let mut inperpreter = Interpreter::new();
        let res = inperpreter.visit(&mut add_node);
        assert_eq!(res, Value::Float(17.0));
    }

    #[test]
    fn unary_op() {
        let five = Value::Float(5.0);
        let two = Value::Float(2.0);
        let minus = Token::new(TokenType::Minus, Value::Char('-'), 1, 1);

        // 5 - -2
        let mut expr_node = Node::BinOp(
            // left
            Box::new(Node::Num(five)),
            // op
            minus.clone(),
            // right
            Box::new(Node::UnaryOp(
                // op
                minus.clone(),
                // right
                Box::new(Node::UnaryOp(minus, Box::new(Node::Num(two)))),
            )),
        );

        let mut inperpreter = Interpreter::new();
        let res = inperpreter.visit(&mut expr_node);
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
        let mut parser = Parser::new(lexer);
        let mut tree = parser.parse();
        let mut interpreter = Interpreter::new();
        interpreter.visit(&mut tree);

        let mut expected = CallStack::new();
        let mut ar = ActivationRecord::new(String::from("VarTest"), ARType::Program, 1);
        ar.set(String::from("number"), Value::Integer(2));
        ar.set(String::from("a"), Value::Integer(2));
        ar.set(String::from("_x"), Value::Integer(11));
        ar.set(String::from("c"), Value::Integer(27));
        ar.set(String::from("b"), Value::Integer(25));
        expected.push(ar);
        assert_eq!(interpreter.call_stack, expected);
    }
}
