# proto-rs

A `.proto` file parser for Rust, supporting proto2, proto3, and editions syntax.

This is a Rust port of [github.com/emicklei/proto](https://github.com/emicklei/proto) — a Go library for parsing Protocol Buffer definition files.

## Features

- Full proto2 and proto3 syntax support
- Edition syntax support
- Complete AST with position tracking
- Visitor pattern for AST traversal
- Walk function for recursive element filtering
- Zero external dependencies
- Comment preservation (inline, leading, C-style, C++ style)

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
proto-parser = { git = "https://github.com/lasorda/proto-rs" }
```

### Parse a `.proto` file

```rust
use proto_parser::{Parser, Element};

let src = r#"
syntax = "proto3";
package example;

message Person {
    string name = 1;
    int32 age = 2;
    repeated string tags = 3;
}
"#;

let mut parser = Parser::new(src);
let proto = parser.parse().unwrap();

for element in &proto.elements {
    match element {
        Element::Message(m) => println!("Message: {}", m.name),
        Element::Package(p) => println!("Package: {}", p.name),
        _ => {}
    }
}
```

### Use the Visitor pattern

```rust
use proto_parser::{Visitor, Message, NormalField};

struct FieldCounter {
    count: usize,
}

impl Visitor for FieldCounter {
    fn visit_normal_field(&mut self, _f: &NormalField) {
        self.count += 1;
    }
}
```

### Walk the AST

```rust
use proto_parser::{Parser, Element, visitor};
use std::cell::RefCell;
use std::rc::Rc;

let mut parser = Parser::new(r#"syntax = "proto3"; message Foo { string name = 1; }"#);
let proto = parser.parse().unwrap();

let messages = Rc::new(RefCell::new(Vec::new()));
let msgs = messages.clone();
let mut handlers: Vec<visitor::Handler> = vec![Box::new(move |e| {
    if let Element::Message(m) = e {
        msgs.borrow_mut().push(m.name.clone());
    }
})];
visitor::walk(&proto, &mut handlers);
// messages now contains ["Foo"]
```

## Supported AST Types

| Type | Description |
|------|-------------|
| `Syntax` | `syntax = "proto3";` |
| `Edition` | `edition = "2023";` |
| `Import` | `import "other.proto";` |
| `Package` | `package foo.bar;` |
| `ProtoOption` | `option java_package = "com.example";` |
| `Message` | Message definitions (also used for `extend`) |
| `Enum` / `EnumField` | Enum definitions and values |
| `NormalField` | Regular fields (including `repeated`, `optional`, `required`) |
| `MapField` | `map<K, V>` fields |
| `Oneof` / `OneofField` | Oneof definitions |
| `Service` / `Rpc` | Service and RPC definitions |
| `Reserved` | Reserved field numbers/names |
| `Extensions` | Extension ranges (proto2) |
| `Group` | Group fields (proto2) |
| `Comment` | Line (`//`) and block (`/* */`) comments |

## Go → Rust Design Mapping

| Go | Rust |
|----|------|
| `Visitee` interface + `[]Visitee` | `Element` enum + `Vec<Element>` |
| `Parent Visitee` | Omitted — use walk stack |
| `error` return | `Result<T, ProtoError>` |
| `text/scanner.Scanner` | Custom `Scanner` struct |
| Embedded `*Field` | `FieldCommon` as named field |
| `NoopVisitor` struct | Trait default methods |
| `WithMessage(func)` handlers | `Box<dyn FnMut(&Element)>` |

## License

MIT — same as the original Go library.
