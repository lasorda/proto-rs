use crate::ast::comment::Comment;
use crate::error::Result;
use crate::parser::Parser;
use crate::position::Position;
use crate::token::{self, Token};

/// Import kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportKind {
    Default,
    Weak,
    Public,
}

/// Import holds a filename to another .proto definition.
#[derive(Debug, Clone)]
pub struct Import {
    pub position: Position,
    pub comment: Option<Comment>,
    pub filename: String,
    pub kind: ImportKind,
    pub inline_comment: Option<Comment>,
}

impl Import {
    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        let (_, tok, lit) = p.next();
        match tok {
            Token::Weak => {
                self.kind = ImportKind::Weak;
                return self.parse(p);
            }
            Token::Public => {
                self.kind = ImportKind::Public;
                return self.parse(p);
            }
            Token::Ident => {
                let (unquoted, _) = token::unquote(&lit);
                self.filename = unquoted;
            }
            _ => {
                return Err(p.unexpected(&lit, "import classifier weak|public|quoted"));
            }
        }
        Ok(())
    }
}
