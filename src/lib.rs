//! A .proto file parser (proto2, proto3, editions).
//!
//! Rust port of [github.com/emicklei/proto](https://github.com/emicklei/proto).

pub mod ast;
pub mod error;
pub mod parser;
pub mod position;
pub mod scanner;
pub mod token;
pub mod visitor;

// Re-exports for convenience.
pub use ast::comment::Comment;
pub use ast::edition::Edition;
pub use ast::enum_field::{Enum, EnumField};
pub use ast::extensions::Extensions;
pub use ast::field::{FieldCommon, MapField, NormalField};
pub use ast::group::Group;
pub use ast::import::{Import, ImportKind};
pub use ast::literal::{Literal, LiteralMap, NamedLiteral};
pub use ast::message::Message;
pub use ast::oneof::{Oneof, OneofField};
pub use ast::option::ProtoOption;
pub use ast::package::Package;
pub use ast::proto::Proto;
pub use ast::range::Range;
pub use ast::reserved::Reserved;
pub use ast::service::{Rpc, Service};
pub use ast::syntax::Syntax;
pub use ast::Element;
pub use error::ProtoError;
pub use parser::Parser;
pub use position::Position;
pub use visitor::Visitor;
