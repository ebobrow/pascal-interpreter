use crate::ast::*;
use crate::interpreter::NodeVisitor;
use crate::tokens::Value;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct CallStack {
    records: Vec<ActivationRecord>,
}

impl CallStack {
    pub fn new() -> Self {
        CallStack {
            records: Vec::new(),
        }
    }

    pub fn push(&mut self, ar: ActivationRecord) {
        self.records.push(ar);
    }

    pub fn pop(&mut self) -> Option<ActivationRecord> {
        self.records.pop()
    }

    pub fn peek(&mut self) -> Option<&mut ActivationRecord> {
        self.records.last_mut()
    }
}

#[derive(Debug, PartialEq)]
pub enum ARType {
    Program,
    Procedure,
}

#[derive(Debug, PartialEq)]
pub struct ActivationRecord {
    name: String,
    type_: ARType,
    nesting_level: usize,
    members: HashMap<String, Value>,
}

impl ActivationRecord {
    pub fn new(name: String, type_: ARType, nesting_level: usize) -> Self {
        ActivationRecord {
            name,
            type_,
            nesting_level,
            members: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: Value) {
        self.members.insert(key, value);
    }

    pub fn get(&self, key: String) -> Option<&Value> {
        self.members.get(&key)
    }
}

pub struct SymbolTableBuilder {
    symtab: SymbolTable,
}

impl NodeVisitor for SymbolTableBuilder {
    fn visit_num(&mut self, _: &mut Num) -> Value {
        Value::None
    }

    fn visit_bin_op(&mut self, bin_op: &mut BinOp) -> Value {
        self.visit(&mut bin_op.left);
        self.visit(&mut bin_op.right)
    }

    fn visit_unary_op(&mut self, unary_op: &mut UnaryOp) -> Value {
        self.visit(&mut unary_op.expr)
    }

    fn visit_compound(&mut self, compound: &mut Compound) -> Value {
        for child in &mut compound.children {
            self.visit(child);
        }

        Value::None
    }

    fn visit_assign(&mut self, assign: &mut Assign) -> Value {
        let var_name = assign.left.value.expect_string();
        self.symtab.lookup(var_name, false).unwrap();

        self.visit(&mut assign.right);

        Value::None
    }

    fn visit_var(&mut self, var: &mut Var) -> Value {
        let var_name = var.value.expect_string();
        self.symtab
            .lookup(var_name.clone(), false)
            // TODO: Use error class
            .unwrap_or_else(|| panic!("Use of undeclared variable: {}", var_name));
        Value::None
    }

    fn visit_program(&mut self, program: &mut Program) -> Value {
        self.visit_block(&mut program.block);
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
        let type_symbol = self
            .symtab
            .lookup(var_decl.type_node.value.expect_string(), false)
            .unwrap();
        let var_name = var_decl.var_node.value.expect_string();
        let var_symbol = VarSymbol::new(var_name, type_symbol.clone());

        self.symtab.insert(Symbol::Var(Box::new(var_symbol)));

        Value::None
    }

    fn visit_type(&mut self, _: &mut Type) -> Value {
        Value::None
    }

    fn visit_procedure_decl(&mut self, _: &mut ProcedureDecl) -> Value {
        Value::None
    }

    fn visit_procedure_call(&mut self, _: &mut ProcedureCall) -> Value {
        Value::None
    }
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
    pub name: String,
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
    pub formal_params: Vec<VarSymbol>,
    pub block_ast: Option<Box<Block>>,
}

impl ProcedureSymbol {
    pub fn new(name: String, formal_params: Vec<VarSymbol>) -> Self {
        ProcedureSymbol {
            name,
            formal_params,
            block_ast: None,
        }
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
        let mut tree = parser.parse();
        let mut symtab_builder = SymbolTableBuilder {
            symtab: SymbolTable::new("global".to_string(), 1, None),
        };
        symtab_builder.visit(&mut tree);

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
        let mut tree = parser.parse();
        let mut symtab_builder = SymbolTableBuilder {
            symtab: SymbolTable::new("global".to_string(), 1, None),
        };
        symtab_builder.visit(&mut tree);
    }
}
