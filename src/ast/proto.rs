use crate::ast::edition::Edition;
use crate::ast::enum_field::{self, maybe_scan_inline_comment};
use crate::ast::import::{Import, ImportKind};
use crate::ast::message::Message;
use crate::ast::option::ProtoOption;
use crate::ast::package::Package;
use crate::ast::service::Service;
use crate::ast::syntax::Syntax;
use crate::ast::{self, Element};
use crate::error::Result;
use crate::parser::Parser;
use crate::token::{self, Token};

/// Proto represents a .proto definition.
#[derive(Debug, Clone)]
pub struct Proto {
    pub filename: String,
    pub elements: Vec<Element>,
}

impl Proto {
    /// Returns a slice of all elements.
    pub fn elements(&self) -> &[Element] {
        &self.elements
    }
}

/// Parse a complete .proto definition source.
pub fn parse_proto(proto: &mut Proto, p: &mut Parser) -> Result<()> {
    loop {
        let (pos, tok, lit) = p.next();
        match tok {
            _ if token::is_comment(&lit) => {
                if let Some(com) = ast::merge_or_return_comment(&mut proto.elements, &lit, pos) {
                    proto.elements.push(Element::Comment(com));
                }
            }
            Token::OptionKw => {
                let comment =
                    ast::take_last_comment_if_ends_on_line(&mut proto.elements, pos.line - 1);
                let mut o = ProtoOption {
                    position: pos,
                    comment,
                    name: String::new(),
                    constant: Default::default(),
                    is_embedded: false,
                    inline_comment: None,
                };
                o.parse(p)?;
                proto.elements.push(Element::Option(o));
            }
            Token::Syntax => {
                let comment =
                    ast::take_last_comment_if_ends_on_line(&mut proto.elements, pos.line - 1);
                let mut s = Syntax {
                    position: pos,
                    comment,
                    value: String::new(),
                    inline_comment: None,
                };
                s.parse(p)?;
                proto.elements.push(Element::Syntax(s));
            }
            Token::Edition => {
                let comment =
                    ast::take_last_comment_if_ends_on_line(&mut proto.elements, pos.line - 1);
                let mut e = Edition {
                    position: pos,
                    comment,
                    value: String::new(),
                    inline_comment: None,
                };
                e.parse(p)?;
                proto.elements.push(Element::Edition(e));
            }
            Token::Import => {
                let comment =
                    ast::take_last_comment_if_ends_on_line(&mut proto.elements, pos.line - 1);
                let mut im = Import {
                    position: pos,
                    comment,
                    filename: String::new(),
                    kind: ImportKind::Default,
                    inline_comment: None,
                };
                im.parse(p)?;
                proto.elements.push(Element::Import(im));
            }
            Token::Enum => {
                let comment =
                    ast::take_last_comment_if_ends_on_line(&mut proto.elements, pos.line - 1);
                let mut e = enum_field::Enum {
                    position: pos,
                    comment,
                    name: String::new(),
                    elements: Vec::new(),
                };
                e.parse(p)?;
                proto.elements.push(Element::Enum(e));
            }
            Token::Service => {
                let comment =
                    ast::take_last_comment_if_ends_on_line(&mut proto.elements, pos.line - 1);
                let mut s = Service {
                    position: pos,
                    comment,
                    name: String::new(),
                    elements: Vec::new(),
                };
                s.parse(p)?;
                proto.elements.push(Element::Service(s));
            }
            Token::Package => {
                let comment =
                    ast::take_last_comment_if_ends_on_line(&mut proto.elements, pos.line - 1);
                let mut pkg = Package {
                    position: pos,
                    comment,
                    name: String::new(),
                    inline_comment: None,
                };
                pkg.parse(p)?;
                proto.elements.push(Element::Package(pkg));
            }
            Token::Message => {
                let comment =
                    ast::take_last_comment_if_ends_on_line(&mut proto.elements, pos.line - 1);
                let mut msg = Message {
                    position: pos,
                    comment,
                    name: String::new(),
                    is_extend: false,
                    elements: Vec::new(),
                };
                msg.parse(p)?;
                proto.elements.push(Element::Message(msg));
            }
            Token::Extend => {
                let comment =
                    ast::take_last_comment_if_ends_on_line(&mut proto.elements, pos.line - 1);
                let mut msg = Message {
                    position: pos,
                    comment,
                    name: String::new(),
                    is_extend: true,
                    elements: Vec::new(),
                };
                msg.parse(p)?;
                proto.elements.push(Element::Message(msg));
            }
            Token::Semicolon => {
                maybe_scan_inline_comment(p, &mut proto.elements);
            }
            Token::Eof => {
                return Ok(());
            }
            _ => {
                return Err(p.unexpected(
                    &lit,
                    ".proto element {comment|option|import|syntax|enum|service|package|message}",
                ));
            }
        }
    }
}
