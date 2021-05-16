use crate::ast::{
    Assign, BinOp, Block, Compound, Num, ProcedureDecl, Program, Type, UnaryOp, Var, VarDecl,
};
use crate::interpreter::NodeVisitor;
use crate::symbols::{BuiltinTypeSymbol, Symbol, SymbolTable, VarSymbol};
use crate::tokens::Value;

pub struct SemanticAnalyzer {
    symtab: SymbolTable,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symtab: SymbolTable::new(),
        }
    }

    pub fn print_symbols(&self) {
        println!("Semantic analyzer symbols: {:#?}", self.symtab);
    }
}

impl NodeVisitor for SemanticAnalyzer {
    fn visit_num(&self, _: &Num) -> Value {
        Value::None
    }

    fn visit_bin_op(&mut self, op: &BinOp) -> Value {
        self.visit(&op.left);
        self.visit(&op.right);
        Value::None
    }

    fn visit_unary_op(&mut self, __op: &UnaryOp) -> Value {
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

    fn visit_var(&self, var: &Var) -> Value {
        let var_name = var.value.expect_string();
        self.symtab
            .lookup(var_name.clone())
            .unwrap_or_else(|| panic!("Identifier not found: {}", var_name));

        Value::None
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

    fn visit_var_decl(&mut self, var_decl: &VarDecl) {
        let type_symbol = self
            .symtab
            .lookup(var_decl.type_node.value.expect_string())
            .unwrap();

        let var_name = var_decl.var_node.value.expect_string();
        let var_symbol = VarSymbol::new(var_name.clone(), type_symbol.clone());
        if let Some(_) = self.symtab.lookup(var_name.clone()) {
            panic!("Duplicate identifier: {}", var_name);
        }
        self.symtab.insert(Symbol::Var(Box::new(var_symbol)));
    }

    fn visit_type(&self, _: &Type) {}

    fn visit_procedure_decl(&self, _: &ProcedureDecl) {}
}
