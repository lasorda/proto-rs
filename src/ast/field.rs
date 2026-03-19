use crate::ast::comment::Comment;
use crate::ast::option::ProtoOption;
use crate::error::Result;
use crate::parser::Parser;
use crate::position::Position;
use crate::token::Token;

/// FieldCommon holds the shared parts of all field types.
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct FieldCommon {
    pub position: Position,
    pub comment: Option<Comment>,
    pub name: String,
    pub type_name: String,
    pub sequence: i64,
    pub options: Vec<ProtoOption>,
    pub inline_comment: Option<Comment>,
}


impl FieldCommon {
    /// Returns true if the option "deprecated" is set with value "true".
    pub fn is_deprecated(&self) -> bool {
        self.options
            .iter()
            .any(|o| o.name == "deprecated" && o.constant.source == "true")
    }
}

/// NormalField represents a field in a Message.
#[derive(Debug, Clone)]
pub struct NormalField {
    pub field: FieldCommon,
    pub repeated: bool,
    pub optional: bool,
    pub required: bool,
}

impl Default for NormalField {
    fn default() -> Self {
        Self::new()
    }
}

impl NormalField {
    pub fn new() -> Self {
        NormalField {
            field: FieldCommon::default(),
            repeated: false,
            optional: false,
            required: false,
        }
    }

    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        loop {
            let (pos, tok, lit) = p.next_type_name();
            match tok {
                Token::Comment => {
                    let c = Comment::new(pos, &lit);
                    if let Some(ref mut existing) = self.field.inline_comment {
                        existing.merge(&c);
                    } else {
                        self.field.inline_comment = Some(c);
                    }
                }
                Token::Repeated => {
                    self.repeated = true;
                    return self.parse(p);
                }
                Token::Optional => {
                    self.optional = true;
                    return self.parse(p);
                }
                Token::Ident => {
                    self.field.type_name = lit;
                    return parse_field_after_type(&mut self.field, p);
                }
                _ => {
                    return Ok(());
                }
            }
        }
    }
}

/// MapField represents a map entry in a message.
#[derive(Debug, Clone)]
pub struct MapField {
    pub field: FieldCommon,
    pub key_type: String,
}

impl Default for MapField {
    fn default() -> Self {
        Self::new()
    }
}

impl MapField {
    pub fn new() -> Self {
        MapField {
            field: FieldCommon::default(),
            key_type: String::new(),
        }
    }

    pub fn parse(&mut self, p: &mut Parser) -> Result<()> {
        let (_, tok, lit) = p.next();
        if tok != Token::Less {
            return Err(p.unexpected(&lit, "map keyType <"));
        }
        let (_, tok, lit) = p.next_type_name();
        if tok != Token::Ident {
            return Err(p.unexpected(&lit, "map identifier"));
        }
        self.key_type = lit;
        let (_, tok, lit) = p.next();
        if tok != Token::Comma {
            return Err(p.unexpected(&lit, "map type separator ,"));
        }
        let (_, tok, lit) = p.next_type_name();
        if tok != Token::Ident {
            return Err(p.unexpected(&lit, "map valueType identifier"));
        }
        self.field.type_name = lit;
        let (_, tok, lit) = p.next();
        if tok != Token::Greater {
            return Err(p.unexpected(&lit, "map valueType >"));
        }
        parse_field_after_type(&mut self.field, p)
    }
}

/// Shared parsing: fieldName = fieldNumber [options]
pub fn parse_field_after_type(f: &mut FieldCommon, p: &mut Parser) -> Result<()> {
    #[derive(PartialEq)]
    enum Expected {
        Ident,
        Equals,
        Number,
    }
    let mut expected = Expected::Ident;

    loop {
        let (pos, tok, lit) = p.next();
        if tok == Token::Comment {
            let c = Comment::new(pos, &lit);
            if let Some(ref mut existing) = f.inline_comment {
                existing.merge(&c);
            } else {
                f.inline_comment = Some(c);
            }
            continue;
        }
        match expected {
            Expected::Ident => {
                if tok != Token::Ident {
                    if tok.is_keyword() {
                        // allow keyword as field name
                    } else {
                        return Err(p.unexpected(&lit, "field identifier"));
                    }
                }
                f.name = lit;
                expected = Expected::Equals;
            }
            Expected::Equals => {
                if tok != Token::Equals {
                    return Err(p.unexpected(&lit, "field ="));
                }
                expected = Expected::Number;
            }
            Expected::Number => {
                if tok != Token::Number {
                    // Could be a negative number (-)
                    if lit == "-" {
                        p.next_put(pos, tok, lit);
                    } else {
                        return Err(p.unexpected(&lit, "field sequence number"));
                    }
                } else {
                    p.next_put(pos, tok, lit);
                }
                let i = p.next_integer()?;
                f.sequence = i;
                break;
            }
        }
    }

    consume_field_comments(f, p);

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
        f.options.push(o);

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

fn consume_field_comments(f: &mut FieldCommon, p: &mut Parser) {
    loop {
        let (pos, tok, lit) = p.next();
        if tok == Token::Comment {
            let c = Comment::new(pos, &lit);
            if let Some(ref mut existing) = f.inline_comment {
                existing.merge(&c);
            } else {
                f.inline_comment = Some(c);
            }
        } else {
            p.next_put(pos, tok, lit);
            break;
        }
    }
}
