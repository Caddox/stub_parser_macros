use crate::flat_stream::Token;

pub struct AstNode {
    Type: Token,
    child: Option<Box<AstNode>>
}

impl AstNode {
    pub fn new(tok: Token, child: Option<Box<AstNode>>) -> AstNode {
        AstNode {
            Type: tok,
            child: child,
        }
    }

    pub fn add_child(&mut self, child: Box<AstNode>) {
        self.child = Some(child);
    }
}