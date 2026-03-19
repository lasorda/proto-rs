/// Token represents a lexical token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    // Special tokens
    Illegal,
    Eof,
    WhiteSpace,

    // Literals
    Ident,

    // Misc characters
    Semicolon,    // ;
    Colon,        // :
    Equals,       // =
    Quote,        // "
    SingleQuote,  // '
    LeftParen,    // (
    RightParen,   // )
    LeftCurly,    // {
    RightCurly,   // }
    LeftSquare,   // [
    RightSquare,  // ]
    Comment,      // // or /* */
    Less,         // <
    Greater,      // >
    Comma,        // ,
    Dot,          // .

    // Keywords
    Edition,
    Syntax,
    Service,
    Rpc,
    Returns,
    Message,
    Import,
    Package,
    OptionKw, // "option" — avoid conflict with std::option::Option
    Repeated,
    Weak,
    Public,
    Oneof,
    Map,
    Reserved,
    Enum,
    Stream,
    Number,

    // proto2
    Optional,
    Group,
    Extensions,
    Extend,
    Required,
}

impl Token {
    /// Returns true if token is a keyword.
    pub fn is_keyword(self) -> bool {
        matches!(
            self,
            Token::Edition
                | Token::Syntax
                | Token::Service
                | Token::Rpc
                | Token::Returns
                | Token::Message
                | Token::Import
                | Token::Package
                | Token::OptionKw
                | Token::Repeated
                | Token::Weak
                | Token::Public
                | Token::Oneof
                | Token::Map
                | Token::Reserved
                | Token::Enum
                | Token::Stream
                | Token::Number
                | Token::Optional
                | Token::Group
                | Token::Extensions
                | Token::Extend
                | Token::Required
        )
    }
}

/// Returns true if the character is whitespace (space, tab, newline).
pub fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n'
}

/// Returns true if the character is a digit.
pub fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

/// Returns true if the literal is a quoted string (single or double).
pub fn is_string(lit: &str) -> bool {
    if lit == "'" {
        return false;
    }
    (lit.starts_with('"') && lit.ends_with('"'))
        || (lit.starts_with('\'') && lit.ends_with('\''))
}

/// Returns true if the literal is a comment.
pub fn is_comment(lit: &str) -> bool {
    lit.starts_with("//") || lit.starts_with("/*")
}

/// Returns true if the literal is a number (int, float, hex).
/// NaN, Inf, Infinity etc. are NOT considered numbers.
pub fn is_number(lit: &str) -> bool {
    match lit {
        "NaN" | "nan" | "Inf" | "Infinity" | "inf" | "infinity" => return false,
        _ => {}
    }
    if lit.starts_with("0x") || lit.starts_with("0X") {
        return i64::from_str_radix(&lit[2..], 16).is_ok();
    }
    lit.parse::<f64>().is_ok()
}

pub const DOUBLE_QUOTE_CHAR: char = '"';

/// Removes one matching leading and trailing single or double quote.
/// Returns the unquoted string and the quote char used.
pub fn unquote(lit: &str) -> (String, char) {
    let chars: Vec<char> = lit.chars().collect();
    if chars.len() < 2 {
        return (lit.to_string(), DOUBLE_QUOTE_CHAR);
    }
    let first = chars[0];
    let last = chars[chars.len() - 1];
    if first != last {
        return (lit.to_string(), DOUBLE_QUOTE_CHAR);
    }
    if first == '"' || first == '\'' {
        let inner: String = chars[1..chars.len() - 1].iter().collect();
        return (inner, first);
    }
    (lit.to_string(), DOUBLE_QUOTE_CHAR)
}

/// Maps a literal string to a Token.
pub fn as_token(literal: &str) -> Token {
    match literal {
        // delimiters
        ";" => Token::Semicolon,
        ":" => Token::Colon,
        "=" => Token::Equals,
        "\"" => Token::Quote,
        "'" => Token::SingleQuote,
        "(" => Token::LeftParen,
        ")" => Token::RightParen,
        "{" => Token::LeftCurly,
        "}" => Token::RightCurly,
        "[" => Token::LeftSquare,
        "]" => Token::RightSquare,
        "<" => Token::Less,
        ">" => Token::Greater,
        "," => Token::Comma,
        "." => Token::Dot,
        // keywords
        "syntax" => Token::Syntax,
        "edition" => Token::Edition,
        "service" => Token::Service,
        "rpc" => Token::Rpc,
        "returns" => Token::Returns,
        "option" => Token::OptionKw,
        "message" => Token::Message,
        "import" => Token::Import,
        "package" => Token::Package,
        "oneof" => Token::Oneof,
        "map" => Token::Map,
        "reserved" => Token::Reserved,
        "enum" => Token::Enum,
        "repeated" => Token::Repeated,
        "weak" => Token::Weak,
        "public" => Token::Public,
        "stream" => Token::Stream,
        // proto2
        "optional" => Token::Optional,
        "group" => Token::Group,
        "extensions" => Token::Extensions,
        "extend" => Token::Extend,
        "required" => Token::Required,
        _ => {
            if is_number(literal) {
                Token::Number
            } else if is_comment(literal) {
                Token::Comment
            } else {
                Token::Ident
            }
        }
    }
}
