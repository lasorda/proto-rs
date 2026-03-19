pub mod comment;
pub mod edition;
pub mod enum_field;
pub mod extensions;
pub mod field;
pub mod group;
pub mod import;
pub mod literal;
pub mod message;
pub mod oneof;
pub mod option;
pub mod package;
pub mod proto;
pub mod range;
pub mod reserved;
pub mod service;
pub mod syntax;

use crate::position::Position;

/// Element is the sum type for all AST nodes in a .proto file.
#[derive(Debug, Clone)]
pub enum Element {
    Comment(comment::Comment),
    Syntax(syntax::Syntax),
    Edition(edition::Edition),
    Import(import::Import),
    Package(package::Package),
    Option(option::ProtoOption),
    Message(message::Message),
    Enum(enum_field::Enum),
    EnumField(enum_field::EnumField),
    Service(service::Service),
    Rpc(service::Rpc),
    NormalField(field::NormalField),
    MapField(field::MapField),
    OneofField(oneof::OneofField),
    Oneof(oneof::Oneof),
    Reserved(reserved::Reserved),
    Group(group::Group),
    Extensions(extensions::Extensions),
}

impl Element {
    /// Returns a reference to this element's children, if it is a container.
    pub fn children(&self) -> &[Element] {
        match self {
            Element::Message(m) => &m.elements,
            Element::Enum(e) => &e.elements,
            Element::Service(s) => &s.elements,
            Element::Oneof(o) => &o.elements,
            Element::Group(g) => &g.elements,
            Element::Rpc(r) => &r.elements,
            Element::EnumField(ef) => &ef.elements,
            _ => &[],
        }
    }

    /// Returns the position of this element.
    pub fn position(&self) -> &Position {
        match self {
            Element::Comment(c) => &c.position,
            Element::Syntax(s) => &s.position,
            Element::Edition(e) => &e.position,
            Element::Import(i) => &i.position,
            Element::Package(p) => &p.position,
            Element::Option(o) => &o.position,
            Element::Message(m) => &m.position,
            Element::Enum(e) => &e.position,
            Element::EnumField(ef) => &ef.position,
            Element::Service(s) => &s.position,
            Element::Rpc(r) => &r.position,
            Element::NormalField(f) => &f.field.position,
            Element::MapField(f) => &f.field.position,
            Element::OneofField(f) => &f.field.position,
            Element::Oneof(o) => &o.position,
            Element::Reserved(r) => &r.position,
            Element::Group(g) => &g.position,
            Element::Extensions(e) => &e.position,
        }
    }

    /// If this element is a Comment, return a reference to it.
    pub fn as_comment(&self) -> Option<&comment::Comment> {
        if let Element::Comment(c) = self {
            Some(c)
        } else {
            None
        }
    }

    /// If this element is a Comment, return a mutable reference.
    pub fn as_comment_mut(&mut self) -> Option<&mut comment::Comment> {
        if let Element::Comment(c) = self {
            Some(c)
        } else {
            None
        }
    }
}

/// Removes and returns the last element if it is a Comment ending on the given line.
pub fn take_last_comment_if_ends_on_line(
    elements: &mut Vec<Element>,
    line: usize,
) -> Option<comment::Comment> {
    if elements.is_empty() {
        return None;
    }
    if let Some(Element::Comment(c)) = elements.last() {
        if c.has_text_on_line(line) {
            if let Element::Comment(c) = elements.pop().unwrap() {
                return Some(c);
            }
        }
    }
    None
}

/// Creates a new comment and tries to merge it with the last element if applicable.
/// Returns Some(comment) if a new standalone comment was created, None if merged.
#[allow(clippy::ptr_arg)]
pub fn merge_or_return_comment(
    elements: &mut Vec<Element>,
    lit: &str,
    pos: Position,
) -> Option<comment::Comment> {
    let com = comment::Comment::new(pos.clone(), lit);
    if elements.is_empty() {
        return Some(com);
    }
    // last element must be a comment to merge
    if let Some(Element::Comment(last)) = elements.last_mut() {
        // do not merge c-style comments
        if last.c_style {
            return Some(com);
        }
        // last comment has text on previous line
        if !last.has_text_on_line(pos.line - 1) {
            return Some(com);
        }
        last.merge(&com);
        return None;
    }
    Some(com)
}

/// Returns the inline comment for element types that support it.
#[allow(clippy::ptr_arg)]
pub fn set_inline_comment(element: &mut Element, comment: comment::Comment) {
    match element {
        Element::Syntax(s) => s.inline_comment = Some(comment),
        Element::Edition(e) => e.inline_comment = Some(comment),
        Element::Import(i) => i.inline_comment = Some(comment),
        Element::Package(p) => p.inline_comment = Some(comment),
        Element::Option(o) => o.inline_comment = Some(comment),
        Element::NormalField(f) => f.field.inline_comment = Some(comment),
        Element::MapField(f) => f.field.inline_comment = Some(comment),
        Element::OneofField(f) => f.field.inline_comment = Some(comment),
        Element::EnumField(ef) => ef.inline_comment = Some(comment),
        Element::Reserved(r) => r.inline_comment = Some(comment),
        Element::Extensions(e) => e.inline_comment = Some(comment),
        Element::Rpc(r) => r.inline_comment = Some(comment),
        _ => {}
    }
}
