use crate::tokens::{Token, Value};

// TODO: Instead of structs for node types, store data directly in enum
#[derive(Debug)]
pub enum Node {
    BinOp(Box<BinOp>),
    Num(Num),
    UnaryOp(Box<UnaryOp>),
    Compound(Compound),
    Assign(Box<Assign>),
    Var(Var),
    Program(Box<Program>),
    // Block(Box<Block>),
    VarDecl(VarDecl),
    // Type(Type),
    ProcedureDecl(Box<ProcedureDecl>),
    NoOp,
}

#[derive(Debug)]
pub struct Program {
    name: String,
    pub block: Block,
}

impl Program {
    pub fn new(name: String, block: Block) -> Self {
        Program { name, block }
    }
}

#[derive(Debug)]
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

#[derive(Clone, Debug)]
pub struct VarDecl {
    pub var_node: Var,
    pub type_node: Type,
}

impl VarDecl {
    pub fn new(var_node: Var, type_node: Type) -> Self {
        VarDecl {
            var_node,
            type_node,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Type {
    token: Token,
    pub value: Value,
}

impl Type {
    pub fn new(token: Token) -> Self {
        Type {
            value: token.value.clone(),
            token,
        }
    }
}

// TODO: token field seems unnecessary?
#[derive(Clone, Debug)]
pub struct Var {
    token: Token,
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

#[derive(Debug)]
pub struct Assign {
    pub left: Var,
    token: Token,
    op: Token,
    pub right: Node,
}

impl Assign {
    pub fn new(left: Var, op: Token, right: Node) -> Self {
        Assign {
            left,
            token: op.clone(),
            op,
            right,
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct BinOp {
    pub left: Node,
    token: Token,
    pub op: Token,
    pub right: Node,
}

impl BinOp {
    pub fn new(left: Node, op: Token, right: Node) -> Self {
        BinOp {
            left,
            token: op.clone(),
            op,
            right,
        }
    }
}

#[derive(Debug)]
pub struct Num {
    token: Token,
    pub value: Value,
}

impl Num {
    pub fn new(token: Token) -> Self {
        Num {
            value: token.value.clone(),
            token,
        }
    }
}

#[derive(Debug)]
pub struct UnaryOp {
    token: Token,
    pub op: Token,
    pub expr: Node,
}

impl UnaryOp {
    pub fn new(op: Token, expr: Node) -> Self {
        UnaryOp {
            token: op.clone(),
            op,
            expr,
        }
    }
}

#[derive(Debug)]
pub struct ProcedureDecl {
    proc_name: String,
    block_node: Block,
}

impl ProcedureDecl {
    pub fn new(proc_name: String, block_node: Block) -> Self {
        ProcedureDecl {
            proc_name,
            block_node,
        }
    }
}
