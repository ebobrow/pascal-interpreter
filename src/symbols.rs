use crate::ast::*;
use crate::interpreter::NodeVisitor;
use crate::tokens::Value;
use std::collections::HashMap;

pub struct SymbolTableBuilder {
    symtab: SymbolTable,
}

impl SymbolTableBuilder {
    pub fn print_contents(&self) {
        println!("Symbol table contents: {:#?}", self.symtab);
    }
}

impl NodeVisitor for SymbolTableBuilder {
    fn visit_num(&mut self, _: &Num) -> Value {
        Value::None
    }

    fn visit_bin_op(&mut self, bin_op: &BinOp) -> Value {
        self.visit(&bin_op.left);
        self.visit(&bin_op.right)
    }

    fn visit_unary_op(&mut self, unary_op: &UnaryOp) -> Value {
        self.visit(&unary_op.expr)
    }

    fn visit_compound(&mut self, compound: &Compound) {
        for child in &compound.children {
            self.visit(child);
        }
    }

    fn visit_assign(&mut self, assign: &Assign) {
        let var_name = assign.left.value.expect_string();
        self.symtab.lookup(var_name, false).unwrap();

        self.visit(&assign.right);
    }

    fn visit_var(&mut self, var: &Var) -> Value {
        let var_name = var.value.expect_string();
        self.symtab
            .lookup(var_name.clone(), false)
            .unwrap_or_else(|| panic!("Use of undeclared variable: {}", var_name));
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
            .lookup(var_decl.type_node.value.expect_string(), false)
            .unwrap();
        let var_name = var_decl.var_node.value.expect_string();
        let var_symbol = VarSymbol::new(var_name, type_symbol.clone());

        self.symtab.insert(Symbol::Var(Box::new(var_symbol)));
    }

    fn visit_type(&mut self, _: &Type) {}

    fn visit_procedure_decl(&mut self, _: &ProcedureDecl) {}

    fn visit_procedure_call(&mut self, _: &ProcedureCall) {}
}

#[derive(Debug, PartialEq)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
    pub scope_level: usize,
    scope_name: String,
    pub enclosing_scope: Option<Box<SymbolTable>>,
}

impl SymbolTable {
    pub fn new(
        scope_name: String,
        scope_level: usize,
        enclosing_scope: Option<SymbolTable>,
    ) -> Self {
        let mut symtab = SymbolTable {
            symbols: HashMap::new(),
            scope_level,
            scope_name,
            enclosing_scope: enclosing_scope.map(Box::new),
        };
        symtab.init_builtins();
        symtab
    }

    fn init_builtins(&mut self) {
        self.insert(Symbol::Builtin(BuiltinTypeSymbol::new(String::from(
            "INTEGER",
        ))));
        self.insert(Symbol::Builtin(BuiltinTypeSymbol::new(String::from(
            "REAL",
        ))));
    }

    pub fn insert(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol.name(), symbol);
    }

    pub fn lookup(&self, name: String, current_scope_only: bool) -> Option<&Symbol> {
        println!("Looking up {} in scope {}", &name, self.scope_name);
        self.symbols.get(&name).or_else(|| {
            if current_scope_only {
                None
            } else {
                self.enclosing_scope
                    .as_ref()
                    .map(|scope| scope.lookup(name, false))
                    .unwrap_or(None)
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
// TODO: Same as AST, don't need struct
pub enum Symbol {
    Builtin(BuiltinTypeSymbol),
    Var(Box<VarSymbol>),
    Procedure(ProcedureSymbol),
}

impl Symbol {
    fn name(&self) -> String {
        match self {
            Symbol::Builtin(b) => b.name.clone(),
            Symbol::Var(v) => v.name.clone(),
            Symbol::Procedure(p) => p.name.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BuiltinTypeSymbol {
    name: String,
}

impl BuiltinTypeSymbol {
    pub fn new(name: String) -> Self {
        BuiltinTypeSymbol { name }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VarSymbol {
    name: String,
    type_: Symbol,
}

impl VarSymbol {
    pub fn new(name: String, type_: Symbol) -> Self {
        VarSymbol { name, type_ }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcedureSymbol {
    name: String,
    pub params: Vec<VarSymbol>,
}

impl ProcedureSymbol {
    pub fn new(name: String, params: Vec<VarSymbol>) -> Self {
        ProcedureSymbol { name, params }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Lexer, Parser};

    #[test]
    fn basic() {
        let text = "
PROGRAM Part11;
VAR
    x : INTEGER;
    y : REAL;

BEGIN

END.";
        let lexer = Lexer::new(text.to_string());
        let mut parser = Parser::new(lexer);
        let tree = parser.parse();
        let mut symtab_builder = SymbolTableBuilder {
            symtab: SymbolTable::new("global".to_string(), 1, None),
        };
        symtab_builder.visit(&tree);

        let mut expected = SymbolTable::new(String::from("global"), 1, None);
        expected.insert(Symbol::Var(Box::new(VarSymbol::new(
            "x".to_string(),
            Symbol::Builtin(BuiltinTypeSymbol::new("INTEGER".to_string())),
        ))));
        expected.insert(Symbol::Var(Box::new(VarSymbol::new(
            "y".to_string(),
            Symbol::Builtin(BuiltinTypeSymbol::new("REAL".to_string())),
        ))));
        assert_eq!(expected, symtab_builder.symtab);
    }

    #[test]
    #[should_panic(expected = "Use of undeclared variable: b")]
    fn undeclared_var() {
        let text = "
PROGRAM Part11;
VAR
    a : INTEGER;

BEGIN
    a := 2 + b;
END.";
        let lexer = Lexer::new(text.to_string());
        let mut parser = Parser::new(lexer);
        let tree = parser.parse();
        let mut symtab_builder = SymbolTableBuilder {
            symtab: SymbolTable::new("global".to_string(), 1, None),
        };
        symtab_builder.visit(&tree);
    }
}
