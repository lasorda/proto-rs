use crate::ast::comment::Comment;
use crate::error::Result;
use crate::parser::Parser;
use crate::position::Position;
use crate::token::{self, Token};

/// Edition declaration (e.g., `edition = "2023";`).
#[derive(Debug, Clone)]
pub struct Edition {
    pub position: Position,
    pub comment: Option<Comment>,
    pub value: String,
    pub inline_comment: Option<Comment>,
}

impl Edition {
    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        let (_, tok, lit) = p.next();
        if tok != Token::Equals {
            return Err(p.unexpected(&lit, "edition ="));
        }
        let (_, _, lit) = p.next();
        if !token::is_string(&lit) {
            return Err(p.unexpected(&lit, "edition string constant"));
        }
        let (unquoted, _) = token::unquote(&lit);
        self.value = unquoted;
        Ok(())
    }
}
