use crate::position::Position;
use crate::token::{self, Token};

/// Scanner is a custom lexical scanner for .proto files.
/// It replaces Go's `text/scanner.Scanner`.
pub struct Scanner {
    input: Vec<char>,
    pos: usize,
    pub position: Position,
    // track current line/col
    line: usize,
    col: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Scanner {
            input: source.chars().collect(),
            pos: 0,
            position: Position {
                filename: String::new(),
                offset: 0,
                line: 1,
                column: 1,
            },
            line: 1,
            col: 1,
        }
    }

    pub fn set_filename(&mut self, name: &str) {
        self.position.filename = name.to_string();
    }

    /// Peek at current char without consuming.
    pub fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    /// Advance past whitespace characters (low-level, for peek_non_whitespace).
    pub fn skip_whitespace_chars(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Advance one character, updating line/col tracking.
    fn advance(&mut self) -> Option<char> {
        if self.pos >= self.input.len() {
            return None;
        }
        let ch = self.input[self.pos];
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    /// Record position for the current token start.
    fn mark_position(&self) -> Position {
        Position {
            filename: self.position.filename.clone(),
            offset: self.pos,
            line: self.line,
            column: self.col,
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Main scan method — returns (position, token, literal_text).
    pub fn scan(&mut self) -> (Position, Token, String) {
        self.skip_whitespace();

        let pos = self.mark_position();

        let ch = match self.peek() {
            Some(c) => c,
            None => {
                self.position = pos.clone();
                return (pos, Token::Eof, String::new());
            }
        };

        // Single-char tokens
        match ch {
            ';' => {
                self.advance();
                self.position = pos.clone();
                return (pos, Token::Semicolon, ";".into());
            }
            ':' => {
                self.advance();
                self.position = pos.clone();
                return (pos, Token::Colon, ":".into());
            }
            '=' => {
                self.advance();
                self.position = pos.clone();
                return (pos, Token::Equals, "=".into());
            }
            '(' => {
                self.advance();
                self.position = pos.clone();
                return (pos, Token::LeftParen, "(".into());
            }
            ')' => {
                self.advance();
                self.position = pos.clone();
                return (pos, Token::RightParen, ")".into());
            }
            '{' => {
                self.advance();
                self.position = pos.clone();
                return (pos, Token::LeftCurly, "{".into());
            }
            '}' => {
                self.advance();
                self.position = pos.clone();
                return (pos, Token::RightCurly, "}".into());
            }
            '[' => {
                self.advance();
                self.position = pos.clone();
                return (pos, Token::LeftSquare, "[".into());
            }
            ']' => {
                self.advance();
                self.position = pos.clone();
                return (pos, Token::RightSquare, "]".into());
            }
            '<' => {
                self.advance();
                self.position = pos.clone();
                return (pos, Token::Less, "<".into());
            }
            '>' => {
                self.advance();
                self.position = pos.clone();
                return (pos, Token::Greater, ">".into());
            }
            ',' => {
                self.advance();
                self.position = pos.clone();
                return (pos, Token::Comma, ",".into());
            }
            '.' => {
                self.advance();
                self.position = pos.clone();
                return (pos, Token::Dot, ".".into());
            }
            _ => {}
        }

        // Double-quoted string
        if ch == '"' {
            let lit = self.scan_double_quoted_string();
            self.position = pos.clone();
            let tok = token::as_token(&lit);
            return (pos, tok, lit);
        }

        // Single quote — return as single quote token, parser handles it
        if ch == '\'' {
            self.advance();
            self.position = pos.clone();
            return (pos, Token::SingleQuote, "'".into());
        }

        // Comments
        if ch == '/' {
            if let Some(lit) = self.try_scan_comment() {
                self.position = pos.clone();
                return (pos, Token::Comment, lit);
            }
            // Not a comment, just a '/'
            self.advance();
            self.position = pos.clone();
            return (pos, Token::Ident, "/".into());
        }

        // Numbers (including negative sign handled by returning '-' as ident)
        if ch == '-' {
            self.advance();
            self.position = pos.clone();
            return (pos, Token::Ident, "-".into());
        }

        if ch.is_ascii_digit() {
            let lit = self.scan_number();
            self.position = pos.clone();
            let tok = token::as_token(&lit);
            return (pos, tok, lit);
        }

        // Identifiers and keywords
        if ch.is_ascii_alphabetic() || ch == '_' {
            let lit = self.scan_identifier();
            self.position = pos.clone();
            let tok = token::as_token(&lit);
            return (pos, tok, lit);
        }

        // Unknown character
        self.advance();
        self.position = pos.clone();
        (pos, Token::Illegal, ch.to_string())
    }

    fn scan_double_quoted_string(&mut self) -> String {
        let mut s = String::new();
        s.push('"');
        self.advance(); // consume opening "
        loop {
            match self.peek() {
                None => break,
                Some('\\') => {
                    s.push('\\');
                    self.advance();
                    if let Some(esc) = self.peek() {
                        s.push(esc);
                        self.advance();
                    }
                }
                Some('"') => {
                    s.push('"');
                    self.advance();
                    break;
                }
                Some(c) => {
                    s.push(c);
                    self.advance();
                }
            }
        }
        s
    }

    fn try_scan_comment(&mut self) -> Option<String> {
        if self.pos + 1 >= self.input.len() {
            return None;
        }
        let next = self.input[self.pos + 1];
        if next == '/' {
            // C++ style line comment
            let mut s = String::new();
            s.push('/');
            self.advance();
            s.push('/');
            self.advance();
            loop {
                match self.peek() {
                    None | Some('\n') => break,
                    Some(c) => {
                        s.push(c);
                        self.advance();
                    }
                }
            }
            Some(s)
        } else if next == '*' {
            // C-style block comment
            let mut s = String::new();
            s.push('/');
            self.advance();
            s.push('*');
            self.advance();
            loop {
                match self.peek() {
                    None => break,
                    Some('*') => {
                        s.push('*');
                        self.advance();
                        if self.peek() == Some('/') {
                            s.push('/');
                            self.advance();
                            break;
                        }
                    }
                    Some(c) => {
                        s.push(c);
                        self.advance();
                    }
                }
            }
            Some(s)
        } else {
            None
        }
    }

    fn scan_number(&mut self) -> String {
        let mut s = String::new();
        // Check for hex 0x / 0X
        if self.peek() == Some('0') {
            s.push('0');
            self.advance();
            if let Some(c) = self.peek() {
                if c == 'x' || c == 'X' {
                    s.push(c);
                    self.advance();
                    // hex digits
                    while let Some(c) = self.peek() {
                        if c.is_ascii_hexdigit() {
                            s.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    return s;
                }
            }
        }
        // decimal / float
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        // fractional part
        if self.peek() == Some('.') {
            // check that next after dot is a digit (not a dot-accessor)
            if self.pos + 1 < self.input.len() && self.input[self.pos + 1].is_ascii_digit() {
                s.push('.');
                self.advance();
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        s.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }
        // exponent
        if let Some(c) = self.peek() {
            if c == 'e' || c == 'E' {
                s.push(c);
                self.advance();
                if let Some(c) = self.peek() {
                    if c == '+' || c == '-' {
                        s.push(c);
                        self.advance();
                    }
                }
                while let Some(c) = self.peek() {
                    if c.is_ascii_digit() {
                        s.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }
        s
    }

    fn scan_identifier(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        s
    }
}
