use crate::ast::comment::Comment;
use crate::ast::edition::Edition;
use crate::ast::enum_field::{Enum, EnumField};
use crate::ast::extensions::Extensions;
use crate::ast::field::{MapField, NormalField};
use crate::ast::group::Group;
use crate::ast::import::Import;
use crate::ast::message::Message;
use crate::ast::oneof::{Oneof, OneofField};
use crate::ast::option::ProtoOption;
use crate::ast::package::Package;
use crate::ast::proto::Proto;
use crate::ast::reserved::Reserved;
use crate::ast::service::{Rpc, Service};
use crate::ast::syntax::Syntax;
use crate::ast::Element;

/// Visitor trait for dispatching proto elements.
/// All methods have default empty implementations (like Go's NoopVisitor).
#[allow(unused_variables)]
pub trait Visitor {
    fn visit_message(&mut self, m: &Message) {}
    fn visit_service(&mut self, s: &Service) {}
    fn visit_syntax(&mut self, s: &Syntax) {}
    fn visit_package(&mut self, p: &Package) {}
    fn visit_option(&mut self, o: &ProtoOption) {}
    fn visit_import(&mut self, i: &Import) {}
    fn visit_normal_field(&mut self, f: &NormalField) {}
    fn visit_enum_field(&mut self, f: &EnumField) {}
    fn visit_enum(&mut self, e: &Enum) {}
    fn visit_comment(&mut self, c: &Comment) {}
    fn visit_oneof(&mut self, o: &Oneof) {}
    fn visit_oneof_field(&mut self, f: &OneofField) {}
    fn visit_reserved(&mut self, r: &Reserved) {}
    fn visit_rpc(&mut self, r: &Rpc) {}
    fn visit_map_field(&mut self, f: &MapField) {}
    fn visit_group(&mut self, g: &Group) {}
    fn visit_extensions(&mut self, e: &Extensions) {}
    fn visit_edition(&mut self, e: &Edition) {}
}

/// Accept dispatches an element to the appropriate visitor method.
pub fn accept(element: &Element, v: &mut dyn Visitor) {
    match element {
        Element::Comment(c) => v.visit_comment(c),
        Element::Syntax(s) => v.visit_syntax(s),
        Element::Edition(e) => v.visit_edition(e),
        Element::Import(i) => v.visit_import(i),
        Element::Package(p) => v.visit_package(p),
        Element::Option(o) => v.visit_option(o),
        Element::Message(m) => v.visit_message(m),
        Element::Enum(e) => v.visit_enum(e),
        Element::EnumField(ef) => v.visit_enum_field(ef),
        Element::Service(s) => v.visit_service(s),
        Element::Rpc(r) => v.visit_rpc(r),
        Element::NormalField(f) => v.visit_normal_field(f),
        Element::MapField(f) => v.visit_map_field(f),
        Element::OneofField(f) => v.visit_oneof_field(f),
        Element::Oneof(o) => v.visit_oneof(o),
        Element::Reserved(r) => v.visit_reserved(r),
        Element::Group(g) => v.visit_group(g),
        Element::Extensions(e) => v.visit_extensions(e),
    }
}

/// Handler is a function that receives an Element reference.
pub type Handler = Box<dyn FnMut(&Element)>;

/// Walk recursively visits all elements of a Proto and calls each handler.
pub fn walk(proto: &Proto, handlers: &mut [Handler]) {
    walk_elements(&proto.elements, handlers);
}

fn walk_elements(elements: &[Element], handlers: &mut [Handler]) {
    for element in elements {
        for handler in handlers.iter_mut() {
            handler(element);
        }
        let children = element.children();
        if !children.is_empty() {
            walk_elements(children, handlers);
        }
    }
}
