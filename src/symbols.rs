use crate::ast::{Assign, BinOp, Block, Compound, Num, Program, Type, UnaryOp, Var, VarDecl};
use crate::interpreter::NodeVisitor;
use crate::tokens::Value;
use std::collections::HashMap;

pub struct SymbolTableBuilder {
    symtab: SymbolTable,
}

impl SymbolTableBuilder {
    pub fn new() -> Self {
        SymbolTableBuilder {
            symtab: SymbolTable::new(),
        }
    }

    pub fn print_contents(&self) {
        println!("Symbol table contents: {:#?}", self.symtab);
    }
}

impl NodeVisitor for SymbolTableBuilder {
    fn visit_num(&self, _: &Num) -> Value {
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
        self.symtab.lookup(var_name).unwrap();

        self.visit(&assign.right);
    }

    fn visit_var(&self, var: &Var) -> Value {
        let var_name = var.value.expect_string();
        self.symtab
            .lookup(var_name.clone())
            .unwrap_or_else(|| panic!("Use of undeclared variable: {}", var_name));
        Value::None
    }

    fn visit_program(&mut self, program: &Program) -> Value {
        self.visit_block(&program.block);
        Value::None
    }

    fn visit_block(&mut self, block: &Block) {
        for declaration in &block.declarations {
            self.visit_var_decl(declaration);
        }
        self.visit(&block.compound_statement);
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) {
        let type_name = var_decl.type_node.value.expect_string();
        let type_symbol = self.symtab.lookup(type_name).unwrap();
        let var_name = var_decl.var_node.value.expect_string();
        let var_symbol = VarSymbol::new(var_name, type_symbol.clone());
        self.symtab.define(Symbol::Var(Box::new(var_symbol)));
    }

    fn visit_type(&self, _: &Type) {}
}

#[derive(Debug, PartialEq)]
struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    fn new() -> Self {
        let mut symtab = SymbolTable {
            symbols: HashMap::new(),
        };
        symtab.init_builtins();
        symtab
    }

    fn init_builtins(&mut self) {
        self.define(Symbol::Builtin(BuiltinTypeSymbol::new(String::from(
            "INTEGER",
        ))));
        self.define(Symbol::Builtin(BuiltinTypeSymbol::new(String::from(
            "REAL",
        ))));
    }

    fn define(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol.name(), symbol);
    }

    fn lookup(&self, name: String) -> Option<&Symbol> {
        self.symbols.get(&name)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Symbol {
    Builtin(BuiltinTypeSymbol),
    Var(Box<VarSymbol>),
}

impl Symbol {
    fn name(&self) -> String {
        match self {
            Symbol::Builtin(b) => b.name.clone(),
            Symbol::Var(v) => v.name.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct BuiltinTypeSymbol {
    name: String,
}

impl BuiltinTypeSymbol {
    fn new(name: String) -> Self {
        BuiltinTypeSymbol { name }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct VarSymbol {
    name: String,
    type_: Symbol,
}

impl VarSymbol {
    fn new(name: String, type_: Symbol) -> Self {
        VarSymbol { name, type_ }
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
        let mut symtab_builder = SymbolTableBuilder::new();
        symtab_builder.visit(&tree);

        let mut expected = SymbolTable::new();
        expected.define(Symbol::Var(Box::new(VarSymbol::new(
            "x".to_string(),
            Symbol::Builtin(BuiltinTypeSymbol::new("INTEGER".to_string())),
        ))));
        expected.define(Symbol::Var(Box::new(VarSymbol::new(
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
        let mut symtab_builder = SymbolTableBuilder::new();
        symtab_builder.visit(&tree);
    }
}
