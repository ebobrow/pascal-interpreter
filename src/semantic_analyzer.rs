use crate::ast::*;
use crate::error::{ErrorCode, SemanticError};
use crate::interpreter::NodeVisitor;
use crate::symbols::{ProcedureSymbol, Symbol, SymbolTable, VarSymbol};
use crate::tokens::Token;
use crate::tokens::Value;

pub struct SemanticAnalyzer {
    current_scope: SymbolTable,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            current_scope: SymbolTable::new(String::from("global"), 1, None),
        }
    }

    fn error(&self, error_code: ErrorCode, token: Token) {
        SemanticError::new(
            format!("{} -> {}", error_code.to_string(), token),
            error_code,
            token,
        )
        .throw();
    }

    pub fn print_symbols(&self) {
        println!("Semantic analyzer symbols: {:#?}", self.current_scope);
    }
}

impl NodeVisitor for SemanticAnalyzer {
    fn visit_num(&mut self, _: &mut Num) -> Value {
        Value::None
    }

    fn visit_bin_op(&mut self, op: &mut BinOp) -> Value {
        self.visit(&mut op.left);
        self.visit(&mut op.right);
        Value::None
    }

    fn visit_unary_op(&mut self, _: &mut UnaryOp) -> Value {
        Value::None
    }

    fn visit_compound(&mut self, compound: &mut Compound) -> Value {
        for child in &mut compound.children {
            self.visit(child);
        }

        Value::None
    }

    fn visit_assign(&mut self, assign: &mut Assign) -> Value {
        self.visit(&mut assign.right);
        self.visit_var(&mut assign.left);

        Value::None
    }

    fn visit_var(&mut self, var: &mut Var) -> Value {
        let var_name = var.value.expect_string();
        self.current_scope
            .lookup(var_name, false)
            .unwrap_or_else(|| {
                self.error(ErrorCode::IDNotFound, var.token.clone());
                unreachable!()
            });

        Value::None
    }

    fn visit_program(&mut self, program: &mut Program) -> Value {
        println!("ENTER scope: global");
        self.current_scope = SymbolTable::new(String::from("global"), 1, None);
        self.visit_block(&mut program.block);
        self.current_scope = *std::mem::replace(&mut self.current_scope.enclosing_scope, None)
            .unwrap_or_else(|| Box::new(SymbolTable::new(String::new(), 0, None)));
        // self.print_symbols();
        println!("LEAVE scope: global");
        Value::None
    }

    fn visit_block(&mut self, block: &mut Block) -> Value {
        for declaration in &mut block.declarations {
            self.visit(declaration);
        }
        self.visit(&mut block.compound_statement);

        Value::None
    }

    fn visit_var_decl(&mut self, var_decl: &mut VarDecl) -> Value {
        let var_name = var_decl.var_node.value.expect_string();
        if self.current_scope.lookup(var_name.clone(), true).is_some() {
            self.error(ErrorCode::DuplicateID, var_decl.var_node.token.clone());
        }

        self.current_scope
            .insert(Symbol::Var(Box::new(VarSymbol::new(
                var_name,
                self.current_scope
                    .lookup(var_decl.type_node.value.expect_string(), false)
                    .unwrap()
                    .clone(),
            ))));

        Value::None
    }

    fn visit_type(&mut self, _: &mut Type) -> Value {
        Value::None
    }

    fn visit_procedure_decl(&mut self, procedure_decl: &mut ProcedureDecl) -> Value {
        let mut proc_symbol = ProcedureSymbol::new(procedure_decl.proc_name.clone(), Vec::new());
        // self.current_scope
        //     .insert(Symbol::Procedure(proc_symbol.clone()));

        println!("ENTER scope: {}", procedure_decl.proc_name.clone());

        let level = self.current_scope.scope_level + 1;
        let prev_scope = std::mem::replace(
            &mut self.current_scope,
            SymbolTable::new(String::from("tmp"), 0, None),
        );
        self.current_scope =
            SymbolTable::new(procedure_decl.proc_name.clone(), level, Some(prev_scope));

        for param in &procedure_decl.formal_params {
            let var_symbol = VarSymbol::new(
                param.var_node.value.expect_string(),
                self.current_scope
                    .lookup(param.type_node.value.expect_string(), false)
                    .unwrap()
                    .clone(),
            );
            self.current_scope
                .insert(Symbol::Var(Box::new(var_symbol.clone())));
            proc_symbol.formal_params.push(var_symbol);
        }
        proc_symbol.block_ast = Some(Box::new(procedure_decl.block_node.clone()));
        self.current_scope
            .enclosing_scope
            .as_mut()
            .map(|scope| scope.insert(Symbol::Procedure(proc_symbol.clone())));

        self.visit_block(&mut procedure_decl.block_node);

        // self.print_symbols();
        self.current_scope =
            *std::mem::replace(&mut self.current_scope.enclosing_scope, None).unwrap();
        println!("LEAVE scope: {}", procedure_decl.proc_name.clone());

        // proc_symbol.block_ast = Some(Box::new(procedure_decl.block_node.clone()));

        Value::None
    }

    fn visit_procedure_call(&mut self, procedure_call: &mut ProcedureCall) -> Value {
        if let Some(Symbol::Procedure(proc)) = self
            .current_scope
            .lookup(procedure_call.proc_name.clone(), true)
        {
            if proc.formal_params.len() != procedure_call.actual_params.len() {
                self.error(ErrorCode::WrongParamsNum, procedure_call.token.clone());
            }
            for param_node in &mut procedure_call.actual_params {
                self.visit(param_node);
            }
            procedure_call.proc_symbol = match self
                .current_scope
                .lookup(procedure_call.proc_name.clone(), false)
            {
                Some(Symbol::Procedure(s)) => Some(s.clone()),
                _ => panic!(),
            };
        } else {
            println!("{:?}", self.current_scope);
            self.error(ErrorCode::IDNotFound, procedure_call.token.clone());
        }

        Value::None
    }
}
