use crate::ast::comment::Comment;
use crate::error::Result;
use crate::parser::Parser;
use crate::position::Position;
use crate::token::{self, Token};

/// Literal represents intLit, floatLit, strLit, boolLit, or a nested structure.
#[derive(Debug, Clone, Default)]
pub struct Literal {
    pub position: Position,
    pub source: String,
    pub is_string: bool,
    pub quote_rune: Option<char>,
    /// If not None, the entry is actually a comment.
    pub comment: Option<Comment>,
    /// Array literal value.
    pub array: Option<Vec<Literal>>,
    /// Ordered map of named literals (preserves insertion order).
    pub ordered_map: Option<LiteralMap>,
}

/// LiteralMap preserves ordering of key-value pairs.
pub type LiteralMap = Vec<NamedLiteral>;

/// Associates a name with a Literal.
#[derive(Debug, Clone)]
pub struct NamedLiteral {
    pub name: String,
    pub literal: Literal,
    /// True when the name must be printed with a colon suffix.
    pub prints_colon: bool,
}

impl Literal {
    /// Source representation (re-quoted if string).
    pub fn source_representation(&self) -> String {
        if self.is_string {
            let q = self.quote_rune.unwrap_or('"');
            format!("{}{}{}", q, self.source, q)
        } else {
            self.source.clone()
        }
    }

    /// Parse a literal constant value.
    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        let (pos, tok, lit) = p.next();

        // Handle comment inside literal
        if token::is_comment(&lit) {
            let nc = Comment::new(pos.clone(), &lit);
            if let Some(ref mut existing) = self.comment {
                existing.merge(&nc);
            } else {
                self.comment = Some(nc);
            }
            // peek at next token
            let (next_pos, next_tok, next_lit) = p.next();
            if matches!(
                next_tok,
                Token::RightSquare | Token::RightCurly | Token::Semicolon | Token::Comma
            ) {
                p.next_put(next_pos, next_tok, next_lit);
                return Ok(());
            }
            p.next_put(next_pos, next_tok, next_lit);
            return self.parse(p);
        }

        if tok == Token::LeftSquare {
            // Array literal
            let mut array = Vec::new();
            let r = p.peek_non_whitespace();
            if r == Some(']') {
                let arr_pos = p.next().0;
                self.array = Some(array);
                self.is_string = false;
                self.position = arr_pos;
                return Ok(());
            }
            loop {
                let mut e = Literal::default();
                e.parse(p)?;
                // skip comment-only literals
                if e.comment.is_some()
                    && e.source.is_empty()
                    && e.array.is_none()
                    && e.ordered_map.is_none()
                {
                    let (_p, t, _l) = p.next();
                    if t == Token::RightSquare {
                        break;
                    }
                    if t == Token::Comma {
                        continue;
                    }
                    p.next_put(_p, t, _l);
                    continue;
                }
                array.push(e);
                let (_p, t, l) = p.next();
                if t == Token::Comma {
                    continue;
                }
                if t == Token::RightSquare {
                    break;
                }
                if t == Token::Comment {
                    p.next_put(_p, t, l);
                    continue;
                }
                return Err(p.unexpected(&l, ", or ]"));
            }
            self.array = Some(array);
            self.is_string = false;
            self.position = pos;
            return Ok(());
        }

        if tok == Token::LeftCurly {
            self.position = pos;
            self.source = String::new();
            self.is_string = false;
            let constants = parse_aggregate_constants(p)?;
            self.ordered_map = Some(constants);
            return Ok(());
        }

        if lit == "-" {
            // negative number
            self.parse(p)?;
            self.position = pos;
            self.source = format!("-{}", self.source);
            return Ok(());
        }

        let source;
        let iss = token::is_string(&lit);
        if iss {
            let (unquoted, qr) = token::unquote(&lit);
            source = unquoted;
            self.quote_rune = Some(qr);
        } else {
            source = lit;
        }
        self.position = pos;
        self.source = source;
        self.is_string = iss;

        // peek for multiline strings
        loop {
            let (p2, t2, l2) = p.next();
            if token::is_string(&l2) {
                let (line, _) = token::unquote(&l2);
                self.source.push_str(&line);
            } else {
                p.next_put(p2, t2, l2);
                break;
            }
        }
        Ok(())
    }
}

/// Get a literal from a LiteralMap by key.
pub fn literal_map_get<'a>(map: &'a LiteralMap, key: &str) -> Option<&'a Literal> {
    map.iter()
        .find(|nl| nl.name == key)
        .map(|nl| &nl.literal)
}

/// Parse aggregate constants inside { ... }.
pub fn parse_aggregate_constants(p: &mut Parser) -> Result<Vec<NamedLiteral>> {
    let mut list = Vec::new();
    loop {
        let (_pos, tok, lit) = p.next_message_literal_field_name();
        if tok == Token::RightCurly {
            return Ok(list);
        }
        if tok == Token::Semicolon {
            continue;
        }
        if tok == Token::Comment {
            continue;
        }
        if tok == Token::Comma {
            if list.is_empty() {
                return Err(p.unexpected(&lit, "non-empty option aggregate key"));
            }
            continue;
        }
        if tok != Token::Ident && !tok.is_keyword() {
            return Err(p.unexpected(&lit, "option aggregate key"));
        }
        // workaround: string concatenation with previous
        if token::is_string(&lit) && !list.is_empty() {
            let (s, _) = token::unquote(&lit);
            list.last_mut().unwrap().literal.source.push_str(&s);
            continue;
        }
        let key = lit;
        let mut prints_colon = false;
        // expect colon, aggregate, or plain literal
        let (pos2, tok2, lit2) = p.next();
        let (pos3, tok3, lit3) = if tok2 == Token::Colon {
            prints_colon = true;
            p.next()
        } else {
            (pos2, tok2, lit2)
        };
        if tok3 == Token::LeftCurly {
            let nested = parse_aggregate_constants(p)?;
            list.push(NamedLiteral {
                name: key,
                prints_colon,
                literal: Literal {
                    ordered_map: Some(nested),
                    ..Default::default()
                },
            });
            continue;
        }
        // no aggregate, put back token
        p.next_put(pos3, tok3, lit3);
        let mut l = Literal::default();
        l.parse(p)?;
        list.push(NamedLiteral {
            name: key,
            literal: l,
            prints_colon,
        });
    }
}
