use crate::ast::*;
use crate::error::{ErrorCode, ParserError};
use crate::lexer::Lexer;
use crate::tokens::{Token, TokenType};

pub struct Parser {
    lexer: Lexer,
    current_token: Option<Token>,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = Some(lexer.get_next_token());
        Parser {
            lexer,
            current_token,
        }
    }

    fn error(&self, error_code: ErrorCode, token: Token) {
        ParserError::new(format!("{} -> {}", error_code, token), error_code, token).throw();
    }

    fn eat(&mut self, token_type: TokenType) {
        if self.current_token.as_ref().unwrap().type_ == token_type {
            self.current_token = Some(self.lexer.get_next_token());
        } else {
            self.error(
                ErrorCode::UnexpectedToken,
                self.current_token.clone().unwrap(),
            );
        }
    }

    fn factor(&mut self) -> Node {
        let token = self.current_token.clone().unwrap();
        match &token.type_ {
            TokenType::Plus => {
                self.eat(TokenType::Plus);
                Node::UnaryOp(Box::new(UnaryOp::new(token, self.factor())))
            }
            TokenType::Minus => {
                self.eat(TokenType::Minus);
                Node::UnaryOp(Box::new(UnaryOp::new(token, self.factor())))
            }
            TokenType::IntegerConst => {
                self.eat(TokenType::IntegerConst);
                Node::Num(Num::new(token))
            }
            TokenType::RealConst => {
                self.eat(TokenType::RealConst);
                Node::Num(Num::new(token))
            }
            TokenType::LeftParen => {
                self.eat(TokenType::LeftParen);
                let node = self.expr();
                self.eat(TokenType::RightParen);
                node
            }
            _ => Node::Var(self.variable()),
        }
    }

    fn term(&mut self) -> Node {
        let mut node = self.factor();

        while let TokenType::Mul | TokenType::IntegerDiv | TokenType::FloatDiv =
            self.current_token.as_ref().unwrap().type_
        {
            let token = self.current_token.clone().unwrap();
            self.eat(token.clone().type_);
            node = Node::BinOp(Box::new(BinOp::new(node, token, self.factor())));
        }
        node
    }

    fn expr(&mut self) -> Node {
        let mut node = self.term();

        while let TokenType::Plus | TokenType::Minus = self.current_token.as_ref().unwrap().type_ {
            let token = self.current_token.clone().unwrap();
            match token.type_ {
                TokenType::Plus => self.eat(TokenType::Plus),
                TokenType::Minus => self.eat(TokenType::Minus),
                _ => unimplemented!(),
            }
            node = Node::BinOp(Box::new(BinOp::new(node, token, self.term())));
        }
        node
    }

    fn empty(&self) -> Node {
        Node::NoOp
    }

    fn variable(&mut self) -> Var {
        let node = Var::new(self.current_token.clone().unwrap());
        self.eat(TokenType::ID);
        node
    }

    fn assignment_statement(&mut self) -> Node {
        let left = self.variable();
        let token = self.current_token.clone().unwrap();
        self.eat(TokenType::Assign);
        let right = self.expr();
        Node::Assign(Box::new(Assign::new(left, token, right)))
    }

    fn statement(&mut self) -> Node {
        match self.current_token.as_ref().unwrap().type_ {
            TokenType::Begin => self.compound_statement(),
            TokenType::ID => self.assignment_statement(),
            _ => self.empty(),
        }
    }

    fn statement_list(&mut self) -> Vec<Node> {
        let node = self.statement();

        let mut results = vec![node];

        while let TokenType::Semi = self.current_token.as_ref().unwrap().type_ {
            self.eat(TokenType::Semi);
            results.push(self.statement());
        }

        if let TokenType::ID = self.current_token.as_ref().unwrap().type_ {
            self.error(ErrorCode::DuplicateID, self.current_token.clone().unwrap());
        }

        results
    }

    fn compound_statement(&mut self) -> Node {
        self.eat(TokenType::Begin);
        let nodes = self.statement_list();
        self.eat(TokenType::End);

        let mut root = Compound::new();
        for node in nodes {
            root.push_child(node);
        }
        Node::Compound(root)
    }

    fn program(&mut self) -> Node {
        self.eat(TokenType::Program);
        let var_node = self.variable();
        let prog_name = var_node.value.expect_string();
        self.eat(TokenType::Semi);
        let block_node = self.block();
        let program_node = Program::new(prog_name, block_node);
        self.eat(TokenType::Dot);

        Node::Program(Box::new(program_node))
    }

    fn block(&mut self) -> Block {
        let declaration_nodes = self.declarations();
        let compound_statement_node = self.compound_statement();
        Block::new(declaration_nodes, compound_statement_node)
    }

    fn declarations(&mut self) -> Vec<Node> {
        let mut declarations = Vec::new();
        if let TokenType::Var = self.current_token.as_ref().unwrap().type_ {
            self.eat(TokenType::Var);
            while let TokenType::ID = self.current_token.as_ref().unwrap().type_ {
                declarations.append(&mut self.variable_declaration());
                self.eat(TokenType::Semi);
            }
        }
        while let TokenType::Procedure = self.current_token.as_ref().unwrap().type_ {
            declarations.push(Node::ProcedureDecl(Box::new(self.procedure_declaration())))
        }
        declarations
    }

    fn procedure_declaration(&mut self) -> ProcedureDecl {
        self.eat(TokenType::Procedure);
        let proc_name = self.current_token.as_ref().unwrap().value.clone();
        self.eat(TokenType::ID);
        let mut params = Vec::new();

        if let TokenType::LeftParen = self.current_token.as_ref().unwrap().type_ {
            self.eat(TokenType::LeftParen);

            params.extend(self.formal_parameter_list());

            self.eat(TokenType::RightParen);
        }

        self.eat(TokenType::Semi);
        let proc_decl = ProcedureDecl::new(proc_name.to_string(), params, self.block());
        self.eat(TokenType::Semi);
        proc_decl
    }

    fn variable_declaration(&mut self) -> Vec<Node> {
        let mut var_nodes = vec![Var::new(self.current_token.clone().unwrap())];
        self.eat(TokenType::ID);

        while let TokenType::Comma = self.current_token.as_ref().unwrap().type_ {
            self.eat(TokenType::Comma);
            var_nodes.push(Var::new(self.current_token.clone().unwrap()));
            self.eat(TokenType::ID);
        }

        self.eat(TokenType::Colon);

        let type_node = self.type_spec();
        let mut var_declarations = Vec::new();
        for node in var_nodes {
            var_declarations.push(Node::VarDecl(VarDecl::new(node, type_node.clone())));
        }
        var_declarations
    }

    fn type_spec(&mut self) -> Type {
        let token = self.current_token.clone().unwrap();
        if let TokenType::Integer = token.type_ {
            self.eat(TokenType::Integer);
        } else {
            self.eat(TokenType::Real);
        }

        Type::new(token)
    }

    fn formal_parameter_list(&mut self) -> Vec<Param> {
        if let TokenType::ID = self.current_token.as_ref().unwrap().type_ {
            let mut param_nodes = self.formal_parameters();

            while let TokenType::Semi = self.current_token.as_ref().unwrap().type_ {
                self.eat(TokenType::Semi);
                param_nodes.extend(self.formal_parameters());
            }
            param_nodes
        } else {
            Vec::new()
        }
    }

    fn formal_parameters(&mut self) -> Vec<Param> {
        let mut param_nodes = Vec::new();
        let mut param_tokens = vec![self.current_token.clone().unwrap()];

        self.eat(TokenType::ID);
        while let TokenType::Comma = self.current_token.as_ref().unwrap().type_ {
            self.eat(TokenType::Comma);
            param_tokens.push(self.current_token.clone().unwrap());
            self.eat(TokenType::ID);
        }
        self.eat(TokenType::Colon);
        let type_node = self.type_spec();

        for param_token in param_tokens {
            param_nodes.push(Param::new(Var::new(param_token), type_node.clone()));
        }

        param_nodes
    }

    pub fn parse(&mut self) -> Node {
        let node = self.program();
        if let TokenType::EOF = self.current_token.as_ref().unwrap().type_ {
            node
        } else {
            panic!("Stuff after the end");
        }
    }
}
