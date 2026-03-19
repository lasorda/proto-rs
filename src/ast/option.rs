use crate::ast::comment::Comment;
use crate::ast::literal::{self, Literal};
use crate::error::Result;
use crate::parser::Parser;
use crate::position::Position;
use crate::token::Token;

/// ProtoOption is a protoc compiler option.
#[derive(Debug, Clone)]
pub struct ProtoOption {
    pub position: Position,
    pub comment: Option<Comment>,
    pub name: String,
    pub constant: Literal,
    pub is_embedded: bool,
    pub inline_comment: Option<Comment>,
}

impl ProtoOption {
    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        consume_option_comments(self, p);
        self.parse_option_name(p)?;

        // check for =
        let (_pos, tok, lit) = p.next();
        if tok != Token::Equals {
            return Err(p.unexpected(&lit, "option value assignment ="));
        }

        let r = p.peek_non_whitespace();
        if r == Some('{') {
            p.next(); // consume {
            self.parse_aggregate(p)?;
        } else {
            let mut l = Literal::default();
            l.parse(p)?;
            self.constant = l;
        }
        consume_option_comments(self, p);
        Ok(())
    }

    fn parse_option_name(&mut self, p: &mut Parser) -> Result<()> {
        let mut name = String::new();
        loop {
            let (pos, tok, lit) = p.next_ident(true);
            match tok {
                Token::Dot => {
                    name.push('.');
                }
                Token::Ident => {
                    name.push_str(&lit);
                }
                Token::LeftParen => {
                    let dot = if p.peek_non_whitespace() == Some('.') {
                        p.next(); // consume dot
                        "."
                    } else {
                        ""
                    };
                    let (_, tok2, lit2) = p.next_full_ident(true);
                    if tok2 != Token::Ident {
                        return Err(p.unexpected(&lit2, "option name"));
                    }
                    let (_, tok3, _) = p.next();
                    if tok3 != Token::RightParen {
                        return Err(p.unexpected(&lit2, "option full identifier closing )"));
                    }
                    name = format!("{}({}{})", name, dot, lit2);
                }
                _ => {
                    p.next_put(pos, tok, lit);
                    break;
                }
            }
        }
        self.name = name;
        Ok(())
    }

    fn parse_aggregate(&mut self, p: &mut Parser) -> Result<()> {
        let constants = literal::parse_aggregate_constants(p)?;
        self.constant = Literal {
            ordered_map: Some(constants),
            position: self.position.clone(),
            ..Default::default()
        };
        Ok(())
    }
}

pub(crate) fn consume_option_comments(o: &mut ProtoOption, p: &mut Parser) {
    loop {
        let (pos, tok, lit) = p.next();
        if tok == Token::Comment {
            let c = Comment::new(pos, &lit);
            if let Some(ref mut existing) = o.comment {
                existing.merge(&c);
            } else {
                o.comment = Some(c);
            }
        } else {
            p.next_put(pos, tok, lit);
            break;
        }
    }
}
