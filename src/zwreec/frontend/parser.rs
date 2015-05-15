//! The `parser` module contains a lot of useful functionality
//! to parse tokens from the lexer (and creating the parsetree
//! and the ast)
//! its an predictiv parser for a LL(1) grammar

pub use frontend::lexer::Token;
pub use frontend::ast;
pub use frontend::parsetree::{ParseTree, PNode};

/*

----------------------------------------
Grammatik:

LL(1)
------------------------------------------------------------+-----------------------------------
S -> Passage S2                                             |
S2 -> S                                                     |
S2 -> ɛ                                                     |
Passage -> PassageName PassageContent                       | "add passage (name)", simple
PassageContent -> TextPassage PassageContent                | add text as child
PassageContent -> Formatting PassageContent                 |
PassageContent -> ɛ                                         |
Formating -> BoldFormatting                                 | 
Formating -> ItalicFormatting                               |
BoldFormatting -> FormatBold BoldContent FormatBold         | add child bold
BoldContent -> TextPassage BoldContent                      | add text as child
BoldContent -> FormatItalic BoldItalicContent FormatItalic  | add child italic 
BoldContent -> ɛ                                            |
ItalicFormatting -> FormatItalic ItalicContent FormatItalic | add child italic 
ItalicContent -> TextPassage ItalicContent                  | add text as child
ItalicContent -> FormatBold BoldItalicContent FormatBold    | add child bold
ItalicContent -> ɛ                                          | add text as child
BoldItalicContent -> TextPassage                            |
------------------------------------------------------------+-----------------------------------

::Start
Hello World, ''Yeah''.
//Italic// is possible too.

*/

pub fn parse_tokens(tokens: Vec<Token>) -> ast::AST {
    let mut parser: Parser = Parser::new(tokens);    
    parser.parsing();
    parser.ast
}

//==============================
// grammar

#[derive(Debug, Copy, Clone)]
pub enum NonTerminalType {
    S,
    S2,
    Passage,
    PassageContent,
    Formating,
    BoldFormatting,
    ItalicFormatting,
    BoldContent,
    ItalicContent,
    BoldItalicContent
}

//==============================
// parser

struct Parser {
    tree: ParseTree,
    ast: ast::AST,
    stack: Stack,
    ast_stack: Vec<usize>,
    tokens: Vec<Token>,
    lookahead: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tree: ParseTree::new(),
            ast: ast::AST::new(),
            stack: Stack::new(),
            ast_stack: Vec::new(),
            tokens: tokens,
            lookahead: 0
        }
    }

    /// the predictive stack ll(1) parsing routine
    pub fn parsing(&mut self) {
        self.stack.push_start();
        
        while let Some(top) = self.stack.pop() {
            if self.tree.is_terminal(top.to_vec()) {
                self.next_token();
            } else {
                self.apply_grammar(top.to_vec());
            }
        }
        
        self.tree.print();
    }

    /// apply the ll(1) grammar
    /// the match-statement simulates the parsing-table behavior
    /// 
    fn apply_grammar(&mut self, top_path: Vec<usize>) {
        if let Some(token) = self.tokens.get_mut(self.lookahead) {

            let to_add_path: Vec<usize> = top_path.to_vec();

            // the frst item in the tuple is the current state and
            // the snd is the current lookup-token
            let state_first: (NonTerminalType, &Token) = (self.tree.get_non_terminal(top_path).clone(), token);

            let mut new_nodes = Vec::new();
            match state_first {
                (NonTerminalType::S, &Token::TokPassageName(_)) => {
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::Passage));
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::S2));
                },
                (NonTerminalType::S2, &Token::TokPassageName(_)) => {
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::S));
                },
                (NonTerminalType::Passage, &Token::TokPassageName(ref name)) => {
                    let new_token: Token = Token::TokPassageName(name.clone());
                    new_nodes.push(PNode::new_terminal(new_token));
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::PassageContent));

                    // ast
                    self.ast_stack.clear();
                    let ast_count_passages = self.ast.cound_childs(self.ast_stack.to_vec());
                    let new_token2: Token = Token::TokPassageName(name.clone());
                    self.ast.add_passage(new_token2);
                    self.ast_stack.push(ast_count_passages);
                },
                (NonTerminalType::PassageContent, &Token::TokText(ref text)) => {
                    let new_token: Token = Token::TokText(text.clone());
                    new_nodes.push(PNode::new_terminal(new_token));
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::PassageContent));

                    // ast
                    let new_token2: Token = Token::TokText(text.clone());
                    self.ast.add_leaf(&self.ast_stack, new_token2);
                },
                (NonTerminalType::PassageContent, &Token::TokFormatBold) | (NonTerminalType::PassageContent, &Token::TokFormatItalic) => {
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::Formating));
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::PassageContent));
                },
                (NonTerminalType::Formating, &Token::TokFormatBold) => {
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::BoldFormatting));
                },
                (NonTerminalType::Formating, &Token::TokFormatItalic) => {
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::ItalicFormatting));
                },
                (NonTerminalType::BoldFormatting, &Token::TokFormatBold) => {
                    new_nodes.push(PNode::new_terminal(Token::TokFormatBold));
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::BoldContent));
                    new_nodes.push(PNode::new_terminal(Token::TokFormatBold));

                    // ast
                    let ast_count_passages = self.ast.cound_childs(self.ast_stack.to_vec());
                    let ast_token: Token = Token::TokFormatBold;
                    self.ast.add_child(&self.ast_stack, ast_token);
                    self.ast_stack.push(ast_count_passages);
                },
                (NonTerminalType::ItalicFormatting, &Token::TokFormatItalic) => {
                    new_nodes.push(PNode::new_terminal(Token::TokFormatItalic));
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::ItalicContent));
                    new_nodes.push(PNode::new_terminal(Token::TokFormatItalic));

                    // ast
                    let ast_count_passages = self.ast.cound_childs(self.ast_stack.to_vec());
                    let ast_token: Token = Token::TokFormatItalic;
                    self.ast.add_child(&self.ast_stack, ast_token);
                    self.ast_stack.push(ast_count_passages);
                },
                (NonTerminalType::BoldContent, &Token::TokText(ref text)) => {
                    let new_token: Token = Token::TokText(text.clone());
                    new_nodes.push(PNode::new_terminal(new_token));
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::BoldContent));

                    // ast
                    let ast_token: Token = Token::TokText(text.clone());
                    self.ast.add_leaf(&self.ast_stack, ast_token);
                },
                (NonTerminalType::ItalicContent, &Token::TokText(ref text)) => {
                    let new_token: Token = Token::TokText(text.clone());
                    new_nodes.push(PNode::new_terminal(new_token));
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::ItalicContent));

                    // ast
                    let ast_token: Token = Token::TokText(text.clone());
                    self.ast.add_leaf(&self.ast_stack, ast_token);
                },
                (NonTerminalType::BoldContent, &Token::TokFormatItalic) => {
                    new_nodes.push(PNode::new_terminal(Token::TokFormatItalic));
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::BoldItalicContent));
                    new_nodes.push(PNode::new_terminal(Token::TokFormatItalic));
                },
                (NonTerminalType::ItalicContent, &Token::TokFormatBold) => {
                    new_nodes.push(PNode::new_terminal(Token::TokFormatBold));
                    new_nodes.push(PNode::new_non_terminal(NonTerminalType::BoldItalicContent));
                    new_nodes.push(PNode::new_terminal(Token::TokFormatBold));
                },
                (NonTerminalType::BoldItalicContent, &Token::TokText(ref text)) => {
                    let new_token: Token = Token::TokText(text.clone());
                    new_nodes.push(PNode::new_terminal(new_token));
                },

                (NonTerminalType::BoldContent, &Token::TokFormatBold) => {
                    // jump one ast-level higher
                    self.ast_stack.pop();
                },
                (NonTerminalType::ItalicContent, &Token::TokFormatItalic) => {
                    // jump one ast-level higher
                    self.ast_stack.pop();
                },
                _ => {

                }
            }

            // adds the new nodes to the tree
            // and adds the path in the tree to the stack
            let length = new_nodes.len().clone();
            self.tree.add_nodes(new_nodes, &to_add_path);
            self.stack.push_path(length as u8, to_add_path);

        } else {
            // no token left
            // only ɛ-productions could be here
            // these productions will be poped of the stack
        }
    }

    /// sets the lookahead to the next token
    fn next_token(&mut self) {
        self.lookahead = self.lookahead + 1;
    }
}

//==============================
// stack of the parser
struct Stack {
    data: Vec<Vec<usize>>
}

impl Stack {
    pub fn new() -> Stack {
        Stack { data: Vec::new() }
    }

    /// pushs the address of the first startsymbol to the stack
    fn push_start(&mut self) {
        self.data.push(vec![]);
    }

    /// save the path of the nodes in the tree to the stack
    /// the right part of the production
    /// should be on the stack in reverse order
    fn push_path(&mut self, count_elements: u8, to_add_path: Vec<usize>) {
        for i in 0..count_elements {
            let mut path: Vec<usize> = to_add_path.to_vec();
            path.push((count_elements-i-1) as usize);
            self.data.push(path);
        }
    }

    fn pop(&mut self) -> Option<Vec<usize>> {
        self.data.pop()
    }
}

//==============================
//
#[test]
fn it_works() {

}

