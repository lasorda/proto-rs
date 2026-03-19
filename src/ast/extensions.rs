use crate::ast::comment::Comment;
use crate::ast::option::ProtoOption;
use crate::ast::range::{self, Range};
use crate::error::Result;
use crate::parser::Parser;
use crate::position::Position;
use crate::token::Token;

/// Extensions declare that a range of field numbers are available for third-party extensions.
/// Proto2 only.
#[derive(Debug, Clone)]
pub struct Extensions {
    pub position: Position,
    pub comment: Option<Comment>,
    pub ranges: Vec<Range>,
    pub inline_comment: Option<Comment>,
    pub options: Vec<ProtoOption>,
}

impl Extensions {
    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        let list = range::parse_ranges(p)?;
        self.ranges = list;

        // see if there are options
        let (pos, tok, lit) = p.next();
        if tok != Token::LeftSquare {
            p.next_put(pos, tok, lit);
            return Ok(());
        }
        // consume options
        loop {
            let mut o = ProtoOption {
                position: pos.clone(),
                comment: None,
                name: String::new(),
                constant: Default::default(),
                is_embedded: true,
                inline_comment: None,
            };
            o.parse(p)?;
            self.options.push(o);
            let (_pos, tok, lit) = p.next();
            if tok == Token::RightSquare {
                break;
            }
            if tok != Token::Comma {
                return Err(p.unexpected(&lit, "option ,"));
            }
        }
        Ok(())
    }
}
