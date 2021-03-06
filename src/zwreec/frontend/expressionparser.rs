//! The `expressionparser` module parses every expression
//! to an AST (abstract syntax tree).
//! The idea is explained here: http://programmers.stackexchange.com/questions/254074/

//use frontend::ast::*;
use frontend::ast::{ASTNode, NodeDefault};
use frontend::lexer::Token;
use frontend::lexer::Token::*;


pub struct ExpressionParser {
    expr_stack: Vec<ASTNode>,
    oper_stack: Vec<Token>,
}

impl ExpressionParser {
    pub fn parse(node: &mut NodeDefault) {
        let mut expr_parser = ExpressionParser {
            expr_stack: Vec::new(),
            oper_stack: Vec::new(),
        };
        expr_parser.parse_expressions(node);
    }

    /// parse the expression node and creates mutliple ast nodes
    fn parse_expressions(&mut self, node: &mut NodeDefault) {
        node.childs.reverse();
        while let Some(top) = node.childs.pop() {
            match top.category() {
                tok @ TokBoolean  { .. } |
                tok @ TokInt      { .. } |
                tok @ TokString   { .. } |
                tok @ TokFunction { .. } |
                tok @ TokVariable { .. } => {
                    let childs_copy = top.as_default().childs.to_vec();
                    self.expr_stack.push( ASTNode::Default(NodeDefault { category: tok.clone(), childs: childs_copy }) );
                },
                tok @ TokNumOp      { .. } |
                tok @ TokCompOp     { .. } |
                tok @ TokLogOp      { .. } |
                tok @ TokUnaryMinus { .. } => {
                    let length = self.oper_stack.len();

                    // cycle through the oper_stack stack backwards
                    // if the rank of the current operator is <= the top of the
                    // stack, we create a new node
                    // if anybody is good in rust, please refactor this. it
                    // should be:
                    // while(is_ranking_not_higher(oper_stack.top(), tok.clone())) { ...
                    for i in 0..length {
                        let i_rev = length - i - 1;
                        let token: Token = match self.oper_stack.get(i_rev) {
                            Some(tok) => tok.clone(),
                            None      => panic!{"No token in the operators stack."}
                        };
                        if is_ranking_not_higher(token.clone(), tok.clone()) {
                            self.new_operator_node();
                        }
                        
                    }

                    self.oper_stack.push(tok.clone());
                },
                tok @ TokExpression => {
                    // more ugly code.
                    // an expression-node is a child of an expression, if there
                    // where parentheses in the expression. but we don't want
                    // them, so we parse the subexpression in the parentheses

                    // make a copy of the top-node. (becouse node is borrowed)
                    // and then parse it again
                    let childs_copy = top.as_default().childs.to_vec();
                    let mut ast_node = NodeDefault { category: tok.clone(), childs: childs_copy };
                    ExpressionParser::parse(&mut ast_node);

                    if let Some(temp) = ast_node.childs.get(0) {
                        self.expr_stack.push(temp.clone());
                    } else {
                        panic!{"no parsable sub-expression"}
                    }
                },
                _ => ()
            }
        }
        // parse the last elements of the stacks
        // to avoid endless loop we try max stack.len()
        for _ in 0..self.expr_stack.len() {
            if self.expr_stack.len() > 0 {
                self.new_operator_node();
            }
        }
        // multiple operators could be on the stack becouse if the unary ops
        for _ in 0..self.oper_stack.len() {
            if self.oper_stack.len() > 0 {
                self.new_operator_node();
            }
        }

        // finished. so add the root of the expressions as child.
        assert!{self.expr_stack.len() == 1, "Only one expression can be the root. But there are {:?}.", self.expr_stack.len()};
        if let Some(root) = self.expr_stack.pop() {
            node.childs.push(root);
        }
    }

    /// creates a node with an operator on top
    fn new_operator_node(&mut self) {
        if let Some(top_op) = self.oper_stack.pop() {

            let is_unary: bool = match top_op.clone() {
                TokLogOp { op_name: op, .. } => match &*op {
                    "not" => true,
                    "!"   => true,
                    _     => false
                },
                TokUnaryMinus { .. } => true,
                _  => false
            };

            if self.expr_stack.len() > 0 {
                let e2: ASTNode = match self.expr_stack.pop() {
                    Some(tok) => tok,
                    None      => panic!{"Not enough elements on the stack to create a node"}
                };

                let mut new_node: ASTNode;

                if is_unary {
                    new_node = ASTNode::Default(NodeDefault { category: top_op.clone(), childs: vec![e2] });
                } else {
                    let e1: ASTNode = match self.expr_stack.pop() {
                        Some(tok) => tok,
                        None      => panic!{"Missing Node to create binary node"}
                    };
                    new_node = ASTNode::Default(NodeDefault { category: top_op.clone(), childs: vec![e1, e2] });
                }

                self.expr_stack.push( new_node );
            } else {
                // multiple unary operators in a row like "not not true"
                self.oper_stack.push(top_op.clone());
            }
        }
    }
}

/// checks the operatores of two tokens returns true if operator of token1
/// is more important then operator of token2
/// the ranking is set in "operator_precedence"
fn is_ranking_not_higher(token1: Token, token2: Token) -> bool {
    let op1: String = match token1 {
        TokUnaryMinus{ .. } => "_".to_string(),
        TokNumOp     { op_name, .. } |
        TokCompOp    { op_name, .. } |
        TokLogOp     { op_name, .. } => {
            op_name.clone()
        },
        _ => panic!{"not allowed operator"}
    };
    let op2: String = match token2 {
        TokUnaryMinus{ .. } => "_".to_string(),
        TokNumOp     { op_name, .. } |
        TokCompOp    { op_name, .. } |
        TokLogOp     { op_name, .. } => {
            op_name.clone()
        },
        _ => panic!{"not allowed operator"}
    };


    // special handling for the unary operators (two unary operators in a row)
    //let op1_copy: &str = op1.as_slice();
    if (op1 == "_" || op1 == "not" || op1 == "!") &&
        operator_rank(op1.clone()) == operator_rank(op2.clone()) {

        return false
    }

    //
    if operator_rank(op1) >= operator_rank(op2) {
        return true
    }

    false
}

/// ranking of the operators
fn operator_rank(op: String) -> u8 {
    match op.as_ref() {
        "or" | "||"         => 1,
        "and" | "&&"        => 2,
        "is" | "==" | "eq" | "neq" | ">" | "gt" | ">=" | "gte" | "<" | "lt" | "<=" | "lte"
                            => 3,
        "+" | "-"           => 4,
        "*" | "/" | "%"     => 5,
        "_" | "not" | "!"   => 6, // _ is unary minus
        _                   => panic!{"This operator is not implemented"}
    }
}
