use crate::ast::*;
use crate::parser::Parser;
use crate::symbols::{ARType, ActivationRecord, CallStack};
use crate::tokens::{TokenType, Value};

pub trait NodeVisitor {
    fn visit_num(&mut self, num: &mut Num) -> Value;
    fn visit_bin_op(&mut self, bin_op: &mut BinOp) -> Value;
    fn visit_unary_op(&mut self, unary_op: &mut UnaryOp) -> Value;
    fn visit_compound(&mut self, compound: &mut Compound) -> Value;
    fn visit_assign(&mut self, assign: &mut Assign) -> Value;
    fn visit_var(&mut self, var: &mut Var) -> Value;
    fn visit_program(&mut self, program: &mut Program) -> Value;
    fn visit_block(&mut self, block: &mut Block) -> Value;
    fn visit_var_decl(&mut self, var_decl: &mut VarDecl) -> Value;
    fn visit_type(&mut self, type_: &mut Type) -> Value;
    fn visit_procedure_decl(&mut self, procedure_decl: &mut ProcedureDecl) -> Value;
    fn visit_procedure_call(&mut self, procedure_call: &mut ProcedureCall) -> Value;

    fn visit(&mut self, node: &mut Node) -> Value {
        match node {
            Node::BinOp(n) => self.visit_bin_op(n),
            Node::UnaryOp(n) => self.visit_unary_op(n),
            Node::Num(n) => self.visit_num(n),
            Node::Compound(n) => self.visit_compound(n),
            Node::NoOp => Value::None,
            Node::Assign(n) => self.visit_assign(n),
            Node::Var(n) => self.visit_var(n),
            Node::Program(n) => self.visit_program(n),
            Node::VarDecl(n) => self.visit_var_decl(n),
            Node::ProcedureDecl(n) => self.visit_procedure_decl(n),
            Node::ProcedureCall(n) => self.visit_procedure_call(n),
            Node::Block(n) => self.visit_block(n),
        }
    }
}

pub struct Interpreter {
    // tree: Node,
    call_stack: CallStack,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            call_stack: CallStack::new(),
        }
    }

    // pub fn interpret(&mut self) -> Value {
    //     self.visit(&mut self.tree)
    // }
}

impl NodeVisitor for Interpreter {
    fn visit_num(&mut self, num: &mut Num) -> Value {
        num.value.clone()
    }

    // TODO: This isn't quite right
    fn visit_bin_op(&mut self, bin_op: &mut BinOp) -> Value {
        let mut float = false;
        let left = match self.visit(&mut bin_op.left) {
            Value::Integer(l) => l as f32,
            Value::Float(l) => {
                float = true;
                l
            }
            _ => panic!(),
        };
        let right = match self.visit(&mut bin_op.right) {
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

    fn visit_unary_op(&mut self, unary_op: &mut UnaryOp) -> Value {
        match self.visit(&mut unary_op.expr) {
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

    fn visit_compound(&mut self, compound: &mut Compound) -> Value {
        for child in &mut compound.children {
            self.visit(child);
        }

        Value::None
    }

    fn visit_assign(&mut self, assign: &mut Assign) -> Value {
        let value = self.visit(&mut assign.right);
        let ar = self.call_stack.peek().unwrap();
        ar.set(assign.left.value.expect_string(), value);

        Value::None
    }

    fn visit_var(&mut self, var: &mut Var) -> Value {
        let ar = self.call_stack.peek().unwrap();
        ar.get(var.value.expect_string().to_lowercase())
            .unwrap()
            .clone()
    }

    fn visit_program(&mut self, program: &mut Program) -> Value {
        println!("ENTER PROGRAM: {}", &program.name);
        self.call_stack.push(ActivationRecord::new(
            program.name.clone(),
            ARType::Program,
            1,
        ));
        self.visit_block(&mut program.block);
        println!("{:#?}", self.call_stack);
        println!("EXIT PROGRAM: {}", &program.name);
        self.call_stack.pop();
        Value::None
    }

    fn visit_block(&mut self, block: &mut Block) -> Value {
        for declaration in &mut block.declarations {
            self.visit(declaration);
        }
        self.visit(&mut block.compound_statement);

        Value::None
    }

    fn visit_var_decl(&mut self, _: &mut VarDecl) -> Value {
        Value::None
    }

    fn visit_type(&mut self, _: &mut Type) -> Value {
        Value::None
    }

    fn visit_procedure_decl(&mut self, _: &mut ProcedureDecl) -> Value {
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
    use crate::lexer::Lexer;
    use crate::tokens::{Token, TokenType, Value};

    #[test]
    fn binary_ops() {
        let mul = Token::new(TokenType::Mul, Value::Char('*'), 1, 1);
        let plus = Token::new(TokenType::Plus, Value::Char('+'), 1, 1);
        let two = Token::new(TokenType::Integer, Value::Float(2.0), 1, 1);
        let seven = Token::new(TokenType::Integer, Value::Float(7.0), 1, 1);
        let three = Token::new(TokenType::Integer, Value::Float(3.0), 1, 1);

        let mut add_node = Node::BinOp(Box::new(BinOp::new(
            Node::BinOp(Box::new(BinOp::new(
                Node::Num(Num::new(two)),
                mul,
                Node::Num(Num::new(seven)),
            ))),
            plus,
            Node::Num(Num::new(three)),
        )));

        let mut inperpreter = Interpreter::new();
        let res = inperpreter.visit(&mut add_node);
        assert_eq!(res, Value::Float(17.0));
    }

    #[test]
    fn unary_op() {
        let five = Token::new(TokenType::Integer, Value::Float(5.0), 1, 1);
        let two = Token::new(TokenType::Integer, Value::Float(2.0), 1, 1);
        let minus = Token::new(TokenType::Minus, Value::Char('-'), 1, 1);

        // 5 - -2
        let mut expr_node = Node::BinOp(Box::new(BinOp::new(
            Node::Num(Num::new(five)),
            minus.clone(),
            Node::UnaryOp(Box::new(UnaryOp::new(
                minus.clone(),
                Node::UnaryOp(Box::new(UnaryOp::new(minus, Node::Num(Num::new(two))))),
            ))),
        )));

        let mut inperpreter = Interpreter::new();
        let res = inperpreter.visit(&mut expr_node);
        assert_eq!(res, Value::Float(3.0));
    }

    //     #[test]
    //     fn variables() {
    //         let text = "
    // PROGRAM VarTest;

    // BEGIN
    //     BEGIN
    //         number := 2;
    //         a := nUmbEr;
    //         b := 10 * a + 10 * NUMBER DIV 4;
    //         c := a - - b;
    //     END;

    //     _x := 11;
    // END.";

    //         let lexer = Lexer::new(text.to_string());
    //         let parser = Parser::new(lexer);
    //         let mut interpreter = Interpreter::new(parser);
    //         interpreter.interpret();

    //         println!("{:?}", interpreter.call_stack);
    //         assert!(false);
    //         let mut expected = CallStack::new();
    //         let mut ar = ActivationRecord::new(String::from("VarTest"), ARType::Program, 1);
    //         ar.set(String::from("number"), Value::Integer(2));
    //         ar.set(String::from("a"), Value::Integer(2));
    //         ar.set(String::from("_x"), Value::Integer(11));
    //         ar.set(String::from("c"), Value::Integer(27));
    //         ar.set(String::from("b"), Value::Integer(25));
    //         expected.push(ar);
    //         // expected.insert(String::from("number"), Value::Integer(2));
    //         // expected.insert(String::from("a"), Value::Integer(2));
    //         // expected.insert(String::from("_x"), Value::Integer(11));
    //         // expected.insert(String::from("c"), Value::Integer(27));
    //         // expected.insert(String::from("b"), Value::Integer(25));

    //         assert_eq!(interpreter.call_stack, expected);
    //     }
}
