use crate::ast::comment::Comment;
use crate::ast::range::{self, Range};
use crate::error::Result;
use crate::parser::Parser;
use crate::position::Position;
use crate::token::{self, Token};

/// Reserved statements declare field numbers or names that cannot be used.
#[derive(Debug, Clone)]
pub struct Reserved {
    pub position: Position,
    pub comment: Option<Comment>,
    pub ranges: Vec<Range>,
    pub field_names: Vec<String>,
    pub inline_comment: Option<Comment>,
}

impl Reserved {
    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        loop {
            let (pos, tok, lit) = p.next();
            if lit.is_empty() {
                return Err(p.unexpected(&lit, "reserved string or integer"));
            }
            let ch = lit.chars().next().unwrap();
            if token::is_digit(ch) || ch == '-' {
                p.next_put(pos, tok, lit);
                let ranges = range::parse_ranges(p)?;
                self.ranges = ranges;
                continue;
            }
            if token::is_string(&lit) {
                let (s, _) = token::unquote(&lit);
                self.field_names.push(s);
                continue;
            }
            if tok == Token::Semicolon {
                p.next_put(pos, tok, lit);
                break;
            }
        }
        Ok(())
    }
}
