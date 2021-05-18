use crate::ast::*;
use crate::interpreter::NodeVisitor;
use crate::symbols::{ProcedureSymbol, Symbol, SymbolTable, VarSymbol};
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

    pub fn print_symbols(&self) {
        println!("Semantic analyzer symbols: {:#?}", self.current_scope);
    }
}

impl NodeVisitor for SemanticAnalyzer {
    fn visit_num(&mut self, _: &Num) -> Value {
        Value::None
    }

    fn visit_bin_op(&mut self, op: &BinOp) -> Value {
        self.visit(&op.left);
        self.visit(&op.right);
        Value::None
    }

    fn visit_unary_op(&mut self, _: &UnaryOp) -> Value {
        Value::None
    }

    fn visit_compound(&mut self, compound: &Compound) {
        for child in &compound.children {
            self.visit(child);
        }
    }

    fn visit_assign(&mut self, assign: &Assign) {
        self.visit(&assign.right);
        self.visit_var(&assign.left);
    }

    fn visit_var(&mut self, var: &Var) -> Value {
        let var_name = var.value.expect_string();
        self.current_scope
            .lookup(var_name.clone(), false)
            .unwrap_or_else(|| panic!("Identifier not found: `{}`", var_name));

        Value::None
    }

    fn visit_program(&mut self, program: &Program) -> Value {
        println!("ENTER scope: global");
        self.current_scope = SymbolTable::new(String::from("global"), 1, None);
        self.visit_block(&program.block);
        self.current_scope = *std::mem::replace(&mut self.current_scope.enclosing_scope, None)
            .unwrap_or(Box::new(SymbolTable::new(String::from("global"), 1, None)));
        self.print_symbols();
        println!("LEAVE scope: global");
        Value::None
    }

    fn visit_block(&mut self, block: &Block) {
        for declaration in &block.declarations {
            self.visit(declaration);
        }
        self.visit(&block.compound_statement);
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) {
        let var_name = var_decl.var_node.value.expect_string();
        if self.current_scope.lookup(var_name.clone(), true).is_some() {
            panic!("Duplicate identifier found: `{}`", &var_name);
        }

        self.current_scope
            .insert(Symbol::Var(Box::new(VarSymbol::new(
                var_name,
                self.current_scope
                    .lookup(var_decl.type_node.value.expect_string(), false)
                    .unwrap()
                    .clone(),
            ))));
    }

    fn visit_type(&mut self, _: &Type) {}

    fn visit_procedure_decl(&mut self, procedure_decl: &ProcedureDecl) {
        let mut proc_symbol = ProcedureSymbol::new(procedure_decl.proc_name.clone(), Vec::new());

        println!("ENTER scope: {}", procedure_decl.proc_name.clone());

        let level = self.current_scope.scope_level + 1;
        let prev_scope = std::mem::replace(
            &mut self.current_scope,
            SymbolTable::new(String::from("tmp"), 0, None),
        );
        self.current_scope =
            SymbolTable::new(procedure_decl.proc_name.clone(), level, Some(prev_scope));

        for param in &procedure_decl.params {
            let var_symbol = VarSymbol::new(
                param.var_node.value.expect_string(),
                self.current_scope
                    .lookup(param.type_node.value.expect_string(), false)
                    .unwrap()
                    .clone(),
            );
            self.current_scope
                .insert(Symbol::Var(Box::new(var_symbol.clone())));
            proc_symbol.params.push(var_symbol);
        }
        self.current_scope.insert(Symbol::Procedure(proc_symbol));

        self.visit_block(&procedure_decl.block_node);

        self.print_symbols();
        self.current_scope =
            *std::mem::replace(&mut self.current_scope.enclosing_scope, None).unwrap();
        println!("LEAVE scope: {}", procedure_decl.proc_name.clone());
    }
}
