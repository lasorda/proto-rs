use crate::error::{ProtoError, Result};
use crate::position::Position;
use crate::scanner::Scanner;
use crate::token::Token;

/// Buffered token values for lookahead.
#[derive(Debug, Clone)]
pub(crate) struct NextValues {
    pub pos: Position,
    pub tok: Token,
    pub lit: String,
}

/// Parser for .proto files.
pub struct Parser {
    pub(crate) scanner: Scanner,
    pub(crate) buf: Option<NextValues>,
    #[allow(dead_code)]
    pub(crate) debug: bool,
    pub(crate) scanner_errors: Vec<String>,
}

impl Parser {
    /// Creates a new parser from source text.
    pub fn new(source: &str) -> Self {
        Parser {
            scanner: Scanner::new(source),
            buf: None,
            debug: false,
            scanner_errors: Vec::new(),
        }
    }

    /// Creates a new parser with a filename for error reporting.
    pub fn with_filename(source: &str, filename: &str) -> Self {
        let mut p = Self::new(source);
        p.scanner.set_filename(filename);
        p
    }

    /// Parse a complete .proto definition.
    pub fn parse(&mut self) -> Result<crate::ast::proto::Proto> {
        let filename = self.scanner.position.filename.clone();
        let mut proto = crate::ast::proto::Proto {
            filename,
            elements: Vec::new(),
        };
        crate::ast::proto::parse_proto(&mut proto, self)?;
        if !self.scanner_errors.is_empty() {
            let msg = self.scanner_errors.join("\n");
            return Err(ProtoError {
                position: Position::default(),
                message: msg,
            });
        }
        Ok(proto)
    }

    /// Returns the next token, either from the buffer or the scanner.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> (Position, Token, String) {
        if let Some(vals) = self.buf.take() {
            return (vals.pos, vals.tok, vals.lit);
        }
        let (pos, tok, lit) = self.scanner.scan();
        if tok == Token::SingleQuote {
            return self.next_single_quoted_string();
        }
        (pos, tok, lit)
    }

    /// Scans a single-quoted string. Called when ' is encountered.
    fn next_single_quoted_string(&mut self) -> (Position, Token, String) {
        let (pos, tok, lit) = self.scanner.scan();
        if tok == Token::Eof {
            return (pos, Token::Eof, String::new());
        }
        // empty single quoted string ''
        if lit == "'" || tok == Token::SingleQuote {
            return (pos, Token::Ident, "''".into());
        }
        let mut result = lit;
        // scan for tokens until closing single-quote
        loop {
            let (_p, t, l) = self.scanner.scan();
            if t == Token::Eof {
                return (_p, Token::Eof, String::new());
            }
            if l == "'" || t == Token::SingleQuote {
                break;
            }
            result.push_str(&l);
        }
        (pos, Token::Ident, format!("'{}'", result))
    }

    /// Push a token back into the buffer.
    pub fn next_put(&mut self, pos: Position, tok: Token, lit: String) {
        self.buf = Some(NextValues { pos, tok, lit });
    }

    /// Create an error for unexpected token.
    pub fn unexpected(&self, found: &str, expected: &str) -> ProtoError {
        ProtoError {
            position: self.scanner.position.clone(),
            message: format!("found {:?} but expected [{}]", found, expected),
        }
    }

    /// Parse the next integer (handles negative and hex).
    pub fn next_integer(&mut self) -> std::result::Result<i64, ProtoError> {
        let (pos, tok, lit) = self.next();
        if lit == "-" {
            let i = self.next_integer()?;
            return Ok(-i);
        }
        if tok != Token::Number {
            return Err(self.unexpected(&lit, "integer"));
        }
        if lit.starts_with("0x") || lit.starts_with("0X") {
            let without_prefix = &lit[2..];
            i64::from_str_radix(without_prefix, 16).map_err(|_| {
                ProtoError {
                    position: pos,
                    message: format!("invalid hex integer: {}", lit),
                }
            })
        } else {
            lit.parse::<i64>().map_err(|_| {
                ProtoError {
                    position: pos,
                    message: format!("invalid integer: {}", lit),
                }
            })
        }
    }

    /// Consumes tokens which may have one or more dot separators (namespaced idents).
    pub fn next_identifier(&mut self) -> (Position, Token, String) {
        let (pos, tok, lit) = self.next_ident(false);
        if tok == Token::Dot {
            let (pos2, _tok2, lit2) = self.next_ident(false);
            return (pos2, Token::Ident, format!(".{}", lit2));
        }
        (pos, tok, lit)
    }

    pub fn next_message_literal_field_name(&mut self) -> (Position, Token, String) {
        let (pos, tok, lit) = self.next_ident(true);
        if tok == Token::LeftSquare {
            let (_pos2, _tok2, lit2) = self.next_ident(true);
            let _ = self.next(); // consume right square
            return (pos, Token::Ident, lit2);
        }
        (pos, tok, lit)
    }

    /// Implements name resolution for type names.
    pub fn next_type_name(&mut self) -> (Position, Token, String) {
        let (pos, tok, lit) = self.next();
        let start_pos = pos.clone();
        let mut full_lit = lit.clone();
        let mut current_tok = tok;

        // leading dot allowed
        if tok == Token::Dot {
            let (_p, t, l) = self.next();
            current_tok = t;
            full_lit = format!(".{}", l);
        }

        // type can be namespaced more
        loop {
            let r = self.peek_non_whitespace();
            if r != Some('.') {
                break;
            }
            self.next(); // consume dot
            let (_p, _t, l) = self.next();
            full_lit = format!("{}.{}", full_lit, l);
            current_tok = Token::Ident;
        }
        (start_pos, current_tok, full_lit)
    }

    pub fn next_ident(&mut self, keyword_start_allowed: bool) -> (Position, Token, String) {
        let (pos, tok, lit) = self.next();
        if tok != Token::Ident
            && !(tok.is_keyword() && keyword_start_allowed) {
                return (pos, tok, lit);
            }
        let start_pos = pos;
        let mut full_lit = lit;
        // see if identifier is namespaced
        loop {
            let r = self.peek_non_whitespace();
            if r != Some('.') {
                break;
            }
            self.next(); // consume dot
            full_lit.push('.');
            let (p2, t2, l2) = self.next();
            if t2 != Token::Ident && !t2.is_keyword() {
                self.next_put(p2, t2, l2);
                break;
            }
            full_lit.push_str(&l2);
        }
        (start_pos, Token::Ident, full_lit)
    }

    pub fn peek_non_whitespace(&mut self) -> Option<char> {
        self.scanner.skip_whitespace_chars();
        self.scanner.peek()
    }

    /// https://protobuf.dev/reference/protobuf/proto3-spec/
    pub fn next_full_ident(&mut self, keyword_start_allowed: bool) -> (Position, Token, String) {
        let (pos, tok, lit) = self.next();
        if tok != Token::Ident
            && !(tok.is_keyword() && keyword_start_allowed) {
                return (pos, tok, lit);
            }
        let mut full_ident = lit;
        loop {
            let r = self.peek_non_whitespace();
            if r != Some('.') {
                break;
            }
            self.next(); // consume dot
            let (p2, t2, l2) = self.next_full_ident(true);
            if t2 != Token::Ident {
                self.next_put(p2, t2, l2);
                break;
            }
            full_ident = format!("{}.{}", full_ident, l2);
        }
        (pos, Token::Ident, full_ident)
    }
}
