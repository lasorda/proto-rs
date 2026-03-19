use crate::ast::comment::Comment;
use crate::error::Result;
use crate::parser::Parser;
use crate::position::Position;
use crate::token::Token;

/// Package specifies the namespace for all proto elements.
#[derive(Debug, Clone)]
pub struct Package {
    pub position: Position,
    pub comment: Option<Comment>,
    pub name: String,
    pub inline_comment: Option<Comment>,
}

impl Package {
    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        let (_, tok, lit) = p.next_ident(true);
        if tok != Token::Ident
            && !tok.is_keyword() {
                return Err(p.unexpected(&lit, "package identifier"));
            }
        self.name = lit;
        Ok(())
    }
}
