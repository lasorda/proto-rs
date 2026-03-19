use crate::ast::comment::Comment;
use crate::ast::option::ProtoOption;
use crate::ast::{self, Element};
use crate::error::Result;
use crate::parser::Parser;
use crate::position::Position;
use crate::token::Token;

/// Enum definition.
#[derive(Debug, Clone)]
pub struct Enum {
    pub position: Position,
    pub comment: Option<Comment>,
    pub name: String,
    pub elements: Vec<Element>,
}

impl Enum {
    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        let (_, tok, lit) = p.next();
        if tok != Token::Ident
            && !tok.is_keyword() {
                return Err(p.unexpected(&lit, "enum identifier"));
            }
        self.name = lit;
        consume_comment_for(p, &mut self.elements);
        let (_, tok, lit) = p.next();
        if tok != Token::LeftCurly {
            return Err(p.unexpected(&lit, "enum opening {"));
        }
        loop {
            let (pos, tok, lit) = p.next();
            match tok {
                Token::Comment => {
                    if let Some(com) = ast::merge_or_return_comment(&mut self.elements, &lit, pos) {
                        self.elements.push(Element::Comment(com));
                    }
                }
                Token::OptionKw => {
                    let comment = ast::take_last_comment_if_ends_on_line(&mut self.elements, pos.line);
                    let mut o = ProtoOption {
                        position: pos,
                        comment,
                        name: String::new(),
                        constant: Default::default(),
                        is_embedded: false,
                        inline_comment: None,
                    };
                    o.parse(p)?;
                    self.elements.push(Element::Option(o));
                }
                Token::RightCurly | Token::Eof => {
                    if tok != Token::RightCurly {
                        return Err(p.unexpected(&lit, "enum closing }"));
                    }
                    return Ok(());
                }
                Token::Semicolon => {
                    maybe_scan_inline_comment(p, &mut self.elements);
                }
                Token::Reserved => {
                    let comment = ast::take_last_comment_if_ends_on_line(&mut self.elements, pos.line - 1);
                    let mut r = crate::ast::reserved::Reserved {
                        position: pos,
                        comment,
                        ranges: Vec::new(),
                        field_names: Vec::new(),
                        inline_comment: None,
                    };
                    r.parse(p)?;
                    self.elements.push(Element::Reserved(r));
                }
                _ => {
                    p.next_put(pos, tok, lit);
                    let comment = ast::take_last_comment_if_ends_on_line(&mut self.elements, {
                        // get pos line from peeked token
                        let peek_line = if let Some(ref buf) = p.buf {
                            buf.pos.line
                        } else {
                            0
                        };
                        if peek_line > 0 { peek_line - 1 } else { 0 }
                    });
                    let pos = if let Some(ref buf) = p.buf {
                        buf.pos.clone()
                    } else {
                        Position::default()
                    };
                    let mut ef = EnumField {
                        position: pos,
                        comment,
                        name: String::new(),
                        integer: 0,
                        elements: Vec::new(),
                        inline_comment: None,
                    };
                    ef.parse(p)?;
                    self.elements.push(Element::EnumField(ef));
                }
            }
        }
    }
}

/// EnumField is part of the body of an Enum.
#[derive(Debug, Clone)]
pub struct EnumField {
    pub position: Position,
    pub comment: Option<Comment>,
    pub name: String,
    pub integer: i64,
    pub elements: Vec<Element>,
    pub inline_comment: Option<Comment>,
}

impl EnumField {
    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        let (_, tok, lit) = p.next_identifier();
        if tok != Token::Ident
            && !tok.is_keyword() {
                return Err(p.unexpected(&lit, "enum field identifier"));
            }
        self.name = lit;
        let (_, tok, lit) = p.next();
        if tok != Token::Equals {
            return Err(p.unexpected(&lit, "enum field ="));
        }
        let i = p.next_integer()?;
        self.integer = i;
        let (pos, tok, lit) = p.next();
        if tok == Token::LeftSquare {
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
                self.elements.push(Element::Option(o));
                let (_pos, tok, _lit) = p.next();
                if tok == Token::Comma {
                    continue;
                }
                if tok == Token::RightSquare {
                    break;
                }
            }
        } else if tok == Token::Semicolon {
            p.next_put(pos, tok, lit);
        }
        Ok(())
    }

    /// Returns true if the option "deprecated" is set with value "true".
    pub fn is_deprecated(&self) -> bool {
        self.elements.iter().any(|e| {
            if let Element::Option(o) = e {
                o.name == "deprecated" && o.constant.source == "true"
            } else {
                false
            }
        })
    }
}

/// Scans for an inline comment on the current line and attaches it to the last element.
#[allow(clippy::ptr_arg)]
pub(crate) fn maybe_scan_inline_comment(p: &mut Parser, elements: &mut Vec<Element>) {
    let current_line = p.scanner.position.line;
    let (pos, tok, lit) = p.next();
    if tok == Token::Comment && pos.line == current_line && !elements.is_empty() {
        let comment = Comment::new(pos, &lit);
        if let Some(last) = elements.last_mut() {
            ast::set_inline_comment(last, comment);
        }
    } else {
        p.next_put(pos, tok, lit);
    }
}

/// Consume comments before a body element (reads until non-comment found).
pub(crate) fn consume_comment_for(p: &mut Parser, elements: &mut Vec<Element>) {
    let (pos, tok, lit) = p.next();
    if tok == Token::Comment {
        if let Some(com) = ast::merge_or_return_comment(elements, &lit, pos) {
            elements.push(Element::Comment(com));
        }
        consume_comment_for(p, elements);
    } else {
        p.next_put(pos, tok, lit);
    }
}
