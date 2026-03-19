use crate::ast::comment::Comment;
use crate::ast::enum_field::{maybe_scan_inline_comment, consume_comment_for};
use crate::ast::option::ProtoOption;
use crate::ast::{self, Element};
use crate::error::Result;
use crate::parser::Parser;
use crate::position::Position;
use crate::token::{self, Token};

/// Service defines a set of RPC calls.
#[derive(Debug, Clone)]
pub struct Service {
    pub position: Position,
    pub comment: Option<Comment>,
    pub name: String,
    pub elements: Vec<Element>,
}

impl Service {
    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        let (_, tok, lit) = p.next_identifier();
        if tok != Token::Ident
            && !tok.is_keyword() {
                return Err(p.unexpected(&lit, "service identifier"));
            }
        self.name = lit;
        consume_comment_for(p, &mut self.elements);
        let (_, tok, lit) = p.next();
        if tok != Token::LeftCurly {
            return Err(p.unexpected(&lit, "service opening {"));
        }
        loop {
            let (pos, tok, lit) = p.next();
            match tok {
                Token::Comment => {
                    if let Some(com) =
                        ast::merge_or_return_comment(&mut self.elements, &lit, pos)
                    {
                        self.elements.push(Element::Comment(com));
                    }
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
                Token::Rpc => {
                    let comment =
                        ast::take_last_comment_if_ends_on_line(&mut self.elements, pos.line - 1);
                    let mut rpc = Rpc {
                        position: pos,
                        comment,
                        name: String::new(),
                        request_type: String::new(),
                        streams_request: false,
                        returns_type: String::new(),
                        streams_returns: false,
                        elements: Vec::new(),
                        inline_comment: None,
                    };
                    rpc.parse(p)?;
                    self.elements.push(Element::Rpc(rpc));
                    maybe_scan_inline_comment(p, &mut self.elements);
                }
                Token::Semicolon => {
                    maybe_scan_inline_comment(p, &mut self.elements);
                }
                Token::RightCurly => {
                    return Ok(());
                }
                _ => {
                    return Err(p.unexpected(&lit, "service comment|rpc"));
                }
            }
        }
    }
}

/// RPC represents an rpc entry in a service.
#[derive(Debug, Clone)]
pub struct Rpc {
    pub position: Position,
    pub comment: Option<Comment>,
    pub name: String,
    pub request_type: String,
    pub streams_request: bool,
    pub returns_type: String,
    pub streams_returns: bool,
    pub elements: Vec<Element>,
    pub inline_comment: Option<Comment>,
}

impl Rpc {
    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        let (_, tok, lit) = p.next();
        if tok != Token::Ident {
            return Err(p.unexpected(&lit, "rpc method"));
        }
        self.name = lit;

        let (_, tok, lit) = p.next();
        if tok != Token::LeftParen {
            return Err(p.unexpected(&lit, "rpc type opening ("));
        }

        let (_, tok, lit) = p.next_type_name();
        let (tok, lit) = if tok == Token::Stream {
            self.streams_request = true;
            let (_, t, l) = p.next_type_name();
            (t, l)
        } else {
            (tok, lit)
        };
        if tok != Token::Ident {
            return Err(p.unexpected(&lit, "rpc stream | request type"));
        }
        self.request_type = lit;

        let (_, tok, lit) = p.next();
        if tok != Token::RightParen {
            return Err(p.unexpected(&lit, "rpc type closing )"));
        }

        let (_, tok, lit) = p.next();
        if tok != Token::Returns {
            return Err(p.unexpected(&lit, "rpc returns"));
        }

        let (_, tok, lit) = p.next();
        if tok != Token::LeftParen {
            return Err(p.unexpected(&lit, "rpc type opening ("));
        }

        let (_, tok, lit) = p.next_type_name();
        let (tok, lit) = if tok == Token::Stream {
            self.streams_returns = true;
            let (_, t, l) = p.next_type_name();
            (t, l)
        } else {
            (tok, lit)
        };
        if tok != Token::Ident {
            return Err(p.unexpected(&lit, "rpc stream | returns type"));
        }
        self.returns_type = lit;

        let (_, tok, lit) = p.next();
        if tok != Token::RightParen {
            return Err(p.unexpected(&lit, "rpc type closing )"));
        }

        let (pos, tok, lit) = p.next();
        if tok == Token::Semicolon {
            p.next_put(pos, tok, lit);
            return Ok(());
        }
        if tok == Token::LeftCurly {
            // parse options inside rpc body
            loop {
                let (pos, tok, lit) = p.next();
                if tok == Token::RightCurly {
                    break;
                }
                if token::is_comment(&lit) {
                    if let Some(com) =
                        ast::merge_or_return_comment(&mut self.elements, &lit, pos)
                    {
                        self.elements.push(Element::Comment(com));
                    }
                    continue;
                }
                if tok == Token::Semicolon {
                    maybe_scan_inline_comment(p, &mut self.elements);
                    continue;
                }
                if tok == Token::OptionKw {
                    let mut o = ProtoOption {
                        position: pos,
                        comment: None,
                        name: String::new(),
                        constant: Default::default(),
                        is_embedded: false,
                        inline_comment: None,
                    };
                    o.parse(p)?;
                    self.elements.push(Element::Option(o));
                }
            }
        }
        Ok(())
    }
}
