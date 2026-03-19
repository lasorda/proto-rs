use crate::error::Result;
use crate::parser::Parser;
use crate::token::{self};

/// Range specifies a number interval (with special "max" end value).
#[derive(Debug, Clone)]
pub struct Range {
    pub from: i64,
    pub to: i64,
    pub max: bool,
}

impl Range {
    /// Returns the source representation of this range.
    pub fn source_representation(&self) -> String {
        if self.max {
            format!("{} to max", self.from)
        } else if self.from == self.to {
            self.from.to_string()
        } else {
            format!("{} to {}", self.from, self.to)
        }
    }
}

/// Parse ranges for extensions and reserved.
pub fn parse_ranges(p: &mut Parser) -> Result<Vec<Range>> {
    let mut list: Vec<Range> = Vec::new();
    let mut seen_to = false;
    let mut negate = false;
    loop {
        let (pos, tok, lit) = p.next();
        if token::is_string(&lit) {
            return Err(p.unexpected(&lit, "integer, <to> <max>"));
        }
        match lit.as_str() {
            "-" => {
                negate = true;
            }
            "," => {}
            "to" => {
                seen_to = true;
            }
            ";" | "[" => {
                p.next_put(pos, tok, lit);
                break;
            }
            "max" => {
                if !seen_to {
                    return Err(p.unexpected(&lit, "to"));
                }
                let from = list.pop().unwrap();
                list.push(Range {
                    from: from.from,
                    to: 0,
                    max: true,
                });
            }
            _ => {
                let i: i64 = lit.parse().map_err(|_| p.unexpected(&lit, "range integer"))?;
                let i = if negate {
                    negate = false;
                    -i
                } else {
                    i
                };
                if seen_to {
                    if list.is_empty() {
                        return Err(p.unexpected(&lit, "integer"));
                    }
                    let from = list.pop().unwrap();
                    list.push(Range {
                        from: from.from,
                        to: i,
                        max: false,
                    });
                    seen_to = false;
                } else {
                    list.push(Range {
                        from: i,
                        to: i,
                        max: false,
                    });
                }
            }
        }
    }
    Ok(list)
}
