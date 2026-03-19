use crate::ast::comment::Comment;
use crate::ast::enum_field::{consume_comment_for, maybe_scan_inline_comment};
use crate::ast::field::{self, FieldCommon};
use crate::ast::group::Group;
use crate::ast::option::ProtoOption;
use crate::ast::{self, Element};
use crate::error::Result;
use crate::parser::Parser;
use crate::position::Position;
use crate::token::Token;

/// Oneof is a field alternate.
#[derive(Debug, Clone)]
pub struct Oneof {
    pub position: Position,
    pub comment: Option<Comment>,
    pub name: String,
    pub elements: Vec<Element>,
}

impl Oneof {
    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        let (_, tok, lit) = p.next();
        if tok != Token::Ident
            && !tok.is_keyword() {
                return Err(p.unexpected(&lit, "oneof identifier"));
            }
        self.name = lit;
        consume_comment_for(p, &mut self.elements);
        let (_, tok, lit) = p.next();
        if tok != Token::LeftCurly {
            return Err(p.unexpected(&lit, "oneof opening {"));
        }
        loop {
            let (pos, tok, lit) = p.next_type_name();
            match tok {
                Token::Comment => {
                    if let Some(com) =
                        ast::merge_or_return_comment(&mut self.elements, &lit, pos)
                    {
                        self.elements.push(Element::Comment(com));
                    }
                }
                Token::Ident => {
                    let comment =
                        ast::take_last_comment_if_ends_on_line(&mut self.elements, pos.line - 1);
                    let mut f = OneofField {
                        field: FieldCommon {
                            position: pos,
                            type_name: lit,
                            comment,
                            ..Default::default()
                        },
                    };
                    field::parse_field_after_type(&mut f.field, p)?;
                    self.elements.push(Element::OneofField(f));
                }
                Token::Group => {
                    let comment =
                        ast::take_last_comment_if_ends_on_line(&mut self.elements, pos.line - 1);
                    let mut g = Group {
                        position: pos,
                        comment,
                        name: String::new(),
                        optional: false,
                        repeated: false,
                        required: false,
                        sequence: 0,
                        elements: Vec::new(),
                    };
                    g.parse(p)?;
                    self.elements.push(Element::Group(g));
                }
                Token::OptionKw => {
                    let comment =
                        ast::take_last_comment_if_ends_on_line(&mut self.elements, pos.line - 1);
                    let mut opt = ProtoOption {
                        position: pos,
                        comment,
                        name: String::new(),
                        constant: Default::default(),
                        is_embedded: false,
                        inline_comment: None,
                    };
                    opt.parse(p)?;
                    self.elements.push(Element::Option(opt));
                }
                Token::Semicolon => {
                    maybe_scan_inline_comment(p, &mut self.elements);
                }
                _ => {
                    if tok != Token::RightCurly {
                        return Err(p.unexpected(&lit, "oneof closing }"));
                    }
                    return Ok(());
                }
            }
        }
    }
}

/// OneofField is part of Oneof.
#[derive(Debug, Clone)]
pub struct OneofField {
    pub field: FieldCommon,
}
