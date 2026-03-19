use crate::ast::comment::Comment;
use crate::ast::enum_field::{self, consume_comment_for, maybe_scan_inline_comment};
use crate::ast::extensions::Extensions;
use crate::ast::field::{MapField, NormalField};
use crate::ast::group::Group;
use crate::ast::oneof::Oneof;
use crate::ast::option::ProtoOption;
use crate::ast::reserved::Reserved;
use crate::ast::{self, Element};
use crate::error::Result;
use crate::parser::Parser;
use crate::position::Position;
use crate::token::{self, Token};

/// Message consists of a message name and a message body.
#[derive(Debug, Clone)]
pub struct Message {
    pub position: Position,
    pub comment: Option<Comment>,
    pub name: String,
    pub is_extend: bool,
    pub elements: Vec<Element>,
}

impl Message {
    fn group_name(&self) -> &str {
        if self.is_extend {
            "extend"
        } else {
            "message"
        }
    }

    /// Parse expects ident { messageBody
    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        let (_, tok, lit) = p.next_identifier();
        if tok != Token::Ident
            && !tok.is_keyword() {
                return Err(p.unexpected(&lit, &format!("{} identifier", self.group_name())));
            }
        self.name = lit;
        consume_comment_for(p, &mut self.elements);
        let (_, tok, lit) = p.next();
        if tok != Token::LeftCurly {
            return Err(p.unexpected(&lit, &format!("{} opening {{", self.group_name())));
        }
        parse_message_body(p, &mut self.elements)
    }
}

/// Parse elements after {. Consumes the closing }.
pub fn parse_message_body(p: &mut Parser, elements: &mut Vec<Element>) -> Result<()> {
    loop {
        let (pos, tok, lit) = p.next();
        match tok {
            _ if token::is_comment(&lit) => {
                if let Some(com) = ast::merge_or_return_comment(elements, &lit, pos) {
                    elements.push(Element::Comment(com));
                }
            }
            Token::Enum => {
                let comment = ast::take_last_comment_if_ends_on_line(elements, pos.line - 1);
                let mut e = enum_field::Enum {
                    position: pos,
                    comment,
                    name: String::new(),
                    elements: Vec::new(),
                };
                e.parse(p)?;
                elements.push(Element::Enum(e));
            }
            Token::Message => {
                let comment = ast::take_last_comment_if_ends_on_line(elements, pos.line - 1);
                let mut msg = Message {
                    position: pos,
                    comment,
                    name: String::new(),
                    is_extend: false,
                    elements: Vec::new(),
                };
                msg.parse(p)?;
                elements.push(Element::Message(msg));
            }
            Token::OptionKw => {
                let comment = ast::take_last_comment_if_ends_on_line(elements, pos.line - 1);
                let mut o = ProtoOption {
                    position: pos,
                    comment,
                    name: String::new(),
                    constant: Default::default(),
                    is_embedded: false,
                    inline_comment: None,
                };
                o.parse(p)?;
                elements.push(Element::Option(o));
            }
            Token::Oneof => {
                let comment = ast::take_last_comment_if_ends_on_line(elements, pos.line - 1);
                let mut o = Oneof {
                    position: pos,
                    comment,
                    name: String::new(),
                    elements: Vec::new(),
                };
                o.parse(p)?;
                elements.push(Element::Oneof(o));
            }
            Token::Map => {
                let comment = ast::take_last_comment_if_ends_on_line(elements, pos.line - 1);
                let mut f = MapField::new();
                f.field.position = pos;
                f.field.comment = comment;
                f.parse(p)?;
                elements.push(Element::MapField(f));
            }
            Token::Reserved => {
                let comment = ast::take_last_comment_if_ends_on_line(elements, pos.line - 1);
                let mut r = Reserved {
                    position: pos,
                    comment,
                    ranges: Vec::new(),
                    field_names: Vec::new(),
                    inline_comment: None,
                };
                r.parse(p)?;
                elements.push(Element::Reserved(r));
            }
            // proto2: optional|repeated|required
            Token::Optional | Token::Repeated | Token::Required => {
                let prev_tok = tok;
                let (pos2, tok2, lit2) = p.next();
                if tok2 == Token::Group {
                    let comment = ast::take_last_comment_if_ends_on_line(elements, pos2.line - 1);
                    let mut g = Group {
                        position: pos2,
                        comment,
                        name: String::new(),
                        optional: prev_tok == Token::Optional,
                        repeated: prev_tok == Token::Repeated,
                        required: prev_tok == Token::Required,
                        sequence: 0,
                        elements: Vec::new(),
                    };
                    g.parse(p)?;
                    elements.push(Element::Group(g));
                } else {
                    p.next_put(pos2, tok2, lit2);
                    let comment = ast::take_last_comment_if_ends_on_line(elements, pos.line - 1);
                    let mut f = NormalField::new();
                    f.field.position = pos;
                    f.field.comment = comment;
                    f.optional = prev_tok == Token::Optional;
                    f.repeated = prev_tok == Token::Repeated;
                    f.required = prev_tok == Token::Required;
                    f.parse(p)?;
                    elements.push(Element::NormalField(f));
                }
            }
            Token::Group => {
                let comment = ast::take_last_comment_if_ends_on_line(elements, pos.line - 1);
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
                elements.push(Element::Group(g));
            }
            Token::Extensions => {
                let comment = ast::take_last_comment_if_ends_on_line(elements, pos.line - 1);
                let mut e = Extensions {
                    position: pos,
                    comment,
                    ranges: Vec::new(),
                    inline_comment: None,
                    options: Vec::new(),
                };
                e.parse(p)?;
                elements.push(Element::Extensions(e));
            }
            Token::Extend => {
                let comment = ast::take_last_comment_if_ends_on_line(elements, pos.line - 1);
                let mut e = Message {
                    position: pos,
                    comment,
                    name: String::new(),
                    is_extend: true,
                    elements: Vec::new(),
                };
                e.parse(p)?;
                elements.push(Element::Message(e));
            }
            Token::RightCurly | Token::Eof => {
                if tok != Token::RightCurly {
                    return Err(p.unexpected(&lit, "extend|message|group closing }"));
                }
                return Ok(());
            }
            Token::Semicolon => {
                maybe_scan_inline_comment(p, elements);
            }
            _ => {
                // field
                p.next_put(pos.clone(), tok, lit);
                let comment = ast::take_last_comment_if_ends_on_line(elements, pos.line - 1);
                let mut f = NormalField::new();
                f.field.position = pos;
                f.field.comment = comment;
                f.parse(p)?;
                elements.push(Element::NormalField(f));
            }
        }
    }
}
