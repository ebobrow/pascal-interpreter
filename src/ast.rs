use crate::symbols::ProcedureSymbol;
use crate::tokens::{Token, Value};

// TODO: Use multiple enums?
// (ex. Program will never appear in bin_op)
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    BinOp(Box<Node>, Token, Box<Node>),
    Num(Value),
    UnaryOp(Token, Box<Node>),
    Compound(Compound),
    Assign(Var, Token, Box<Node>),
    Var(Var),
    Program(String, Box<Block>),
    // Block(Box<Block>),
    VarDecl(Var, Type),
    // Type(Type),
    ProcedureDecl(String, Box<Block>, Vec<Param>),
    // Param(Param),
    ProcedureCall(ProcedureCall),
    // Block(Box<Block>),
    NoOp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub declarations: Vec<Node>,
    pub compound_statement: Node,
}

impl Block {
    pub fn new(declarations: Vec<Node>, compound_statement: Node) -> Self {
        Block {
            declarations,
            compound_statement,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Type {
    pub value: Value,
}

impl Type {
    pub fn new(token: Token) -> Self {
        Type { value: token.value }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Var {
    pub token: Token,
    pub value: Value,
}

impl Var {
    pub fn new(token: Token) -> Self {
        Var {
            value: token.value.clone(),
            token,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Compound {
    pub children: Vec<Node>,
}

impl Compound {
    pub fn new() -> Self {
        Compound {
            children: Vec::new(),
        }
    }

    pub fn push_child(&mut self, child: Node) {
        self.children.push(child);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub var_node: Var,
    pub type_node: Type,
}

impl Param {
    pub fn new(var_node: Var, type_node: Type) -> Self {
        Param {
            var_node,
            type_node,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureCall {
    pub proc_name: String,
    pub actual_params: Vec<Node>,
    pub token: Token,
    pub proc_symbol: Option<ProcedureSymbol>,
}

impl ProcedureCall {
    pub fn new(proc_name: String, actual_params: Vec<Node>, token: Token) -> Self {
        ProcedureCall {
            proc_name,
            actual_params,
            token,
            proc_symbol: None,
        }
    }
}
