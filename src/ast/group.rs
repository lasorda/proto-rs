use crate::ast::comment::Comment;
use crate::ast::enum_field::consume_comment_for;
use crate::ast::message::parse_message_body;
use crate::ast::Element;
use crate::error::Result;
use crate::parser::Parser;
use crate::position::Position;
use crate::token::Token;

/// Group represents a proto2-only group.
#[derive(Debug, Clone)]
pub struct Group {
    pub position: Position,
    pub comment: Option<Comment>,
    pub name: String,
    pub optional: bool,
    pub repeated: bool,
    pub required: bool,
    pub sequence: i64,
    pub elements: Vec<Element>,
}

impl Group {
    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        let (_, tok, lit) = p.next();
        if tok != Token::Ident
            && !tok.is_keyword() {
                return Err(p.unexpected(&lit, "group name"));
            }
        self.name = lit;
        let (_, tok, lit) = p.next();
        if tok != Token::Equals {
            return Err(p.unexpected(&lit, "group ="));
        }
        let i = p.next_integer()?;
        self.sequence = i;
        consume_comment_for(p, &mut self.elements);
        let (_, tok, lit) = p.next();
        if tok != Token::LeftCurly {
            return Err(p.unexpected(&lit, "group opening {"));
        }
        parse_message_body(p, &mut self.elements)
    }
}
