use proto_parser::*;
use proto_parser::token::*;

#[test]
fn test_unquote_cases() {
    let cases = vec![
        ("thanos", "thanos", '"'),
        ("`bucky`", "`bucky`", '"'),
        ("'nat", "'nat", '"'),
        ("'bruce'", "bruce", '\''),
        ("\"tony\"", "tony", '"'),
        ("\"'\"\"'  -> \"\"\"\"\"\"", "'\"\"'  -> \"\"\"\"\"", '"'),
        ("\"''\"", "''", '"'),
        ("''", "", '\''),
        ("", "", '"'),
    ];
    for (i, (input, expected_output, expected_rune)) in cases.iter().enumerate() {
        let (got, got_rune) = unquote(input);
        assert_eq!(
            got_rune, *expected_rune,
            "[{}] rune: got {:?} want {:?}",
            i, got_rune, expected_rune
        );
        assert_eq!(
            got, *expected_output,
            "[{}] got {:?} want {:?}",
            i, got, expected_output
        );
    }
}

#[test]
fn test_is_number() {
    let cases = vec![
        ("1", true),
        ("1.2", true),
        ("-1.02", true),
        ("a1", false),
        ("0x12", true),
        ("0X77777", true),
        ("NaN", false),
        ("nan", false),
        ("Inf", false),
        ("Infinity", false),
        ("inf", false),
        ("infinity", false),
    ];
    for (i, (input, expected)) in cases.iter().enumerate() {
        let got = is_number(input);
        assert_eq!(got, *expected, "[{}] got {} want {}", i, got, expected);
    }
}

#[test]
fn test_as_token() {
    assert_eq!(as_token(";"), Token::Semicolon);
    assert_eq!(as_token("syntax"), Token::Syntax);
    assert_eq!(as_token("message"), Token::Message);
    assert_eq!(as_token("123"), Token::Number);
    assert_eq!(as_token("// hi"), Token::Comment);
    assert_eq!(as_token("foo"), Token::Ident);
}

#[test]
fn test_syntax() {
    let proto = r#"syntax = "proto";"#;
    let mut p = Parser::new(proto);
    p.next(); // consume "syntax"
    let mut s = proto_parser::ast::syntax::Syntax {
        position: Position::default(),
        comment: None,
        value: String::new(),
        inline_comment: None,
    };
    s.parse(&mut p).unwrap();
    assert_eq!(s.value, "proto");
}

#[test]
fn test_edition() {
    let proto = r#"edition = "1967";"#;
    let mut p = Parser::new(proto);
    let pr = p.parse().unwrap();
    match &pr.elements[0] {
        Element::Edition(e) => {
            assert_eq!(e.value, "1967");
        }
        _ => panic!("expected Edition"),
    }
}

#[test]
fn test_parse_import() {
    let proto = r#"import public "other.proto";"#;
    let mut p = Parser::new(proto);
    p.next(); // consume "import"
    let mut i = proto_parser::ast::import::Import {
        position: Position::default(),
        comment: None,
        filename: String::new(),
        kind: ImportKind::Default,
        inline_comment: None,
    };
    i.parse(&mut p).unwrap();
    assert_eq!(i.filename, "other.proto");
    assert_eq!(i.kind, ImportKind::Public);
}

#[test]
fn test_package_parse_with_reserved_prefix() {
    let want = "rpc.enum.oneof";
    let ident = format!(" {};", want);
    let mut p = Parser::new(&ident);
    let mut pkg = proto_parser::ast::package::Package {
        position: Position::default(),
        comment: None,
        name: String::new(),
        inline_comment: None,
    };
    pkg.parse(&mut p).unwrap();
    assert_eq!(pkg.name, want);
}

#[test]
fn test_scan_ignore_whitespace_digits() {
    let mut p = Parser::new(" 1234 ");
    let (_, _, lit) = p.next();
    assert_eq!(lit, "1234");
}

#[test]
fn test_scan_ignore_whitespace_minus() {
    let mut p = Parser::new(" -1234");
    let (_, _, lit) = p.next();
    assert_eq!(lit, "-");
}

#[test]
fn test_next_identifier() {
    let ident = " aap.noot.mies ";
    let mut p = Parser::new(ident);
    let (_, tok, lit) = p.next_identifier();
    assert_eq!(tok, token::Token::Ident);
    assert_eq!(lit, "aap.noot.mies");
}

#[test]
fn test_next_identifier_with_keyword() {
    let ident = " aap.rpc.mies.enum =";
    let mut p = Parser::new(ident);
    let (_, tok, lit) = p.next_identifier();
    assert_eq!(tok, token::Token::Ident);
    assert_eq!(lit, "aap.rpc.mies.enum");
    let (_, tok, _) = p.next();
    assert_eq!(tok, token::Token::Equals);
}

#[test]
fn test_next_type_name_with_leading_keyword() {
    let ident = " service.me.now";
    let mut p = Parser::new(ident);
    let (_, tok, lit) = p.next_type_name();
    assert_eq!(tok, token::Token::Ident);
    assert_eq!(lit, "service.me.now");
}

#[test]
fn test_next_identifier_no_ident() {
    let ident = "(";
    let mut p = Parser::new(ident);
    let (_, tok, lit) = p.next_identifier();
    assert_eq!(tok, token::Token::LeftParen);
    assert_eq!(lit, "(");
}

#[test]
fn test_parse_comment() {
    let proto = r#"
    // first
	// second

    /*
	ctyle
	multi
	line
    */

    // cpp style single line //

	message test{}
	"#;
    let mut p = Parser::new(proto);
    let pr = p.parse().unwrap();
    let comment_count = pr
        .elements
        .iter()
        .filter(|e| matches!(e, Element::Comment(_)))
        .count();
    assert_eq!(comment_count, 3);
}

#[test]
fn test_parse_comment_with_empty_lines_indent_and_triple_slash() {
    let proto = r#"
	// comment 1
	// comment 2
	//
	// comment 3
	/// comment 4"#;
    let mut p = Parser::new(proto);
    let def = p.parse().unwrap();
    assert_eq!(def.elements.len(), 1);
    match &def.elements[0] {
        Element::Comment(c) => {
            assert_eq!(c.lines.len(), 5);
            assert_eq!(c.lines[4], " comment 4");
            assert_eq!(c.position.line, 2);
            assert!(!c.c_style);
        }
        _ => panic!("expected Comment"),
    }
}

#[test]
fn test_parse_c_style_comment() {
    let proto = r#"
/*comment 1
comment 2

comment 3
  comment 4
*/"#;
    let mut p = Parser::new(proto);
    let def = p.parse().unwrap();
    assert_eq!(def.elements.len(), 1);
    match &def.elements[0] {
        Element::Comment(c) => {
            assert_eq!(c.lines.len(), 6);
            assert_eq!(c.lines[3], "comment 3");
            assert_eq!(c.lines[4], "  comment 4");
            assert!(c.c_style);
        }
        _ => panic!("expected Comment"),
    }
}

#[test]
fn test_parse_c_style_one_line_comment() {
    let proto = "/* comment 1 */";
    let mut p = Parser::new(proto);
    let def = p.parse().unwrap();
    assert_eq!(def.elements.len(), 1);
    match &def.elements[0] {
        Element::Comment(c) => {
            assert_eq!(c.lines.len(), 1);
            assert_eq!(c.lines[0], " comment 1 ");
            assert!(c.c_style);
        }
        _ => panic!("expected Comment"),
    }
}

#[test]
fn test_parse_comment_with_triple_slash() {
    let proto = "\n/// comment 1\n";
    let mut p = Parser::new(proto);
    let def = p.parse().unwrap();
    assert_eq!(def.elements.len(), 1);
    match &def.elements[0] {
        Element::Comment(c) => {
            assert!(c.extra_slash);
            assert_eq!(c.lines[0], " comment 1");
            assert_eq!(c.position.line, 2);
        }
        _ => panic!("expected Comment"),
    }
}

#[test]
fn test_protobuf_issue_4726() {
    let src = r#"syntax = "proto3";

	service SomeService {
		rpc SomeMethod (Whatever) returns (Whatever) {
			option (google.api.http) = {
				delete : "/some/url"
				additional_bindings {
					delete: "/another/url"
				}
			};
		}
	}"#;
    let mut p = Parser::new(src);
    let result = p.parse();
    assert!(result.is_ok(), "parse error: {:?}", result.err());
}

#[test]
fn test_proto_issue_92() {
    let src = r#"syntax = "proto3";

package test;

message Foo {
  .game.Resource one = 1 [deprecated = true];
  repeated .game.sub.Resource two = 2;
  map<string, .game.Resource> three = 3;
}"#;
    let mut p = Parser::new(src);
    let result = p.parse();
    assert!(result.is_ok(), "parse error: {:?}", result.err());
}

#[test]
fn test_parse_single_quotes_strings() {
    let mut p = Parser::new(" 'bohemian','' ");
    let (_, _, lit) = p.next();
    assert_eq!(lit, "'bohemian'");
    let (_, tok, _) = p.next();
    assert_eq!(tok, token::Token::Comma);
    let (_, _, lit) = p.next();
    assert_eq!(lit, "''");
}

#[test]
fn test_proto_issue_132() {
    let src = r#"syntax = "proto3";
package tutorial;
message Person {
  string name = 1;
  int32 id = 0x2;  // Unique ID number for this person.
  string email = 0X3;
}"#;
    let mut p = Parser::new(src);
    let result = p.parse();
    assert!(result.is_ok(), "parse error: {:?}", result.err());
}

#[test]
fn test_reserved_negative_ranges() {
    let mut p = Parser::new("reserved -1;");
    let (_, tok, _) = p.next(); // consume "reserved"
    assert_eq!(tok, Token::Reserved);
    let mut r = proto_parser::ast::reserved::Reserved {
        position: Position::default(),
        comment: None,
        ranges: Vec::new(),
        field_names: Vec::new(),
        inline_comment: None,
    };
    r.parse(&mut p).unwrap();
    assert_eq!(r.ranges[0].source_representation(), "-1");
}

#[test]
fn test_parse_negative_enum() {
    let def = r#"
syntax = "proto3";
package example;

enum Value {
  ZERO = 0;
  reserved -2, -1;
}"#;
    let mut p = Parser::new(def);
    let result = p.parse();
    assert!(result.is_ok(), "parse error: {:?}", result.err());
}

#[test]
fn test_parse_inf_message() {
    let def = r#"
message Inf {
	string field = 1;
}
message NaN {
	string field = 1;
}

message Infinity {
	string field = 1;
}
message ExampelMessage {
	Inf inf_field = 1;
	NaN nan_field = 2;
	Infinity infinity_field = 3;
}
"#;
    let mut p = Parser::new(def);
    let result = p.parse();
    assert!(result.is_ok(), "parse error: {:?}", result.err());
}

#[test]
fn test_full_ident() {
    let cases = vec![
        ("i", Token::Ident),
        ("ident12_", Token::Ident),
        ("ident12_ident42.Ident01_Ident2", Token::Ident),
        ("enum", Token::Enum),
        ("enum_enum", Token::Ident),
    ];
    for (src, expected_tok) in cases {
        let mut p = Parser::new(src);
        let (_, tok, lit) = p.next_full_ident(false);
        assert_eq!(tok, expected_tok, "src={}", src);
        assert_eq!(lit, src, "src={}", src);
    }
}

#[test]
fn test_full_ident_starting_with_keyword() {
    let cases = vec!["service", "enum_service", "message_enum.service"];
    for src in cases {
        let mut p = Parser::new(src);
        let (_, tok, lit) = p.next_full_ident(true);
        assert_eq!(tok, Token::Ident, "src={}", src);
        assert_eq!(lit, src, "src={}", src);
    }
}

#[test]
fn test_comment_association() {
    let src = r#"
	// foo1
	// foo2

	// bar

	syntax = "proto3";

	// baz

	// bat1
	// bat2
	package bat;

	// Oneway is the return type to use for an rpc method if
	// the method should be generated as oneway.
	message Oneway {
	  bool ack = 1;
	}"#;
    let mut p = Parser::new(src);
    let def = p.parse().unwrap();
    assert_eq!(def.elements.len(), 6);
    match &def.elements[4] {
        Element::Package(pkg) => {
            let comment = pkg.comment.as_ref().unwrap();
            assert_eq!(comment.message(), " bat1");
            assert_eq!(comment.lines.len(), 2);
            assert_eq!(comment.lines[1], " bat2");
        }
        _ => panic!("expected Package at index 4"),
    }
    match &def.elements[5] {
        Element::Message(m) => {
            assert_eq!(m.comment.as_ref().unwrap().lines.len(), 2);
        }
        _ => panic!("expected Message at index 5"),
    }
}

#[test]
fn test_comment_in_option_value() {
    let src = r#"syntax = "proto3";
message Foo {
  string bar = 1 [
	// comment
	// me
    deprecated=true
  ];
}"#;
    let mut p = Parser::new(src);
    let def = p.parse().unwrap();
    let msg = match &def.elements[1] {
        Element::Message(m) => m,
        _ => panic!("expected Message"),
    };
    let field = match &msg.elements[0] {
        Element::NormalField(f) => f,
        _ => panic!("expected NormalField"),
    };
    assert!(field.field.is_deprecated());
    let opt = &field.field.options[0];
    assert_eq!(opt.name, "deprecated");
    let comment = opt.comment.as_ref().unwrap();
    assert_eq!(comment.lines.len(), 2);
    assert_eq!(comment.lines[1], " me");
}

#[test]
fn test_parse_service_with_rpc() {
    let src = r#"syntax = "proto3";
service MyService {
    rpc GetUser (GetUserRequest) returns (GetUserResponse);
    rpc ListUsers (ListUsersRequest) returns (stream ListUsersResponse);
}"#;
    let mut p = Parser::new(src);
    let def = p.parse().unwrap();
    let svc = match &def.elements[1] {
        Element::Service(s) => s,
        _ => panic!("expected Service"),
    };
    assert_eq!(svc.name, "MyService");
    let rpc1 = match &svc.elements[0] {
        Element::Rpc(r) => r,
        _ => panic!("expected Rpc"),
    };
    assert_eq!(rpc1.name, "GetUser");
    assert_eq!(rpc1.request_type, "GetUserRequest");
    assert_eq!(rpc1.returns_type, "GetUserResponse");
    assert!(!rpc1.streams_request);
    assert!(!rpc1.streams_returns);

    let rpc2 = match &svc.elements[1] {
        Element::Rpc(r) => r,
        _ => panic!("expected Rpc"),
    };
    assert_eq!(rpc2.name, "ListUsers");
    assert!(rpc2.streams_returns);
}

#[test]
fn test_parse_enum() {
    let src = r#"syntax = "proto3";
enum Status {
    UNKNOWN = 0;
    ACTIVE = 1;
    INACTIVE = 2;
}"#;
    let mut p = Parser::new(src);
    let def = p.parse().unwrap();
    let e = match &def.elements[1] {
        Element::Enum(e) => e,
        _ => panic!("expected Enum"),
    };
    assert_eq!(e.name, "Status");
    let fields: Vec<&proto_parser::EnumField> = e
        .elements
        .iter()
        .filter_map(|el| {
            if let Element::EnumField(ef) = el {
                Some(ef)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].name, "UNKNOWN");
    assert_eq!(fields[0].integer, 0);
    assert_eq!(fields[2].name, "INACTIVE");
    assert_eq!(fields[2].integer, 2);
}

#[test]
fn test_parse_message_with_fields() {
    let src = r#"syntax = "proto3";
message Person {
    string name = 1;
    int32 age = 2;
    repeated string tags = 3;
    map<string, int32> scores = 4;
}"#;
    let mut p = Parser::new(src);
    let def = p.parse().unwrap();
    let msg = match &def.elements[1] {
        Element::Message(m) => m,
        _ => panic!("expected Message"),
    };
    assert_eq!(msg.name, "Person");
    // Check fields
    let fields: Vec<&Element> = msg
        .elements
        .iter()
        .filter(|e| !matches!(e, Element::Comment(_)))
        .collect();
    assert_eq!(fields.len(), 4);

    // name field
    match fields[0] {
        Element::NormalField(f) => {
            assert_eq!(f.field.name, "name");
            assert_eq!(f.field.type_name, "string");
            assert_eq!(f.field.sequence, 1);
            assert!(!f.repeated);
        }
        _ => panic!("expected NormalField"),
    }
    // repeated tags
    match fields[2] {
        Element::NormalField(f) => {
            assert_eq!(f.field.name, "tags");
            assert!(f.repeated);
        }
        _ => panic!("expected NormalField"),
    }
    // map field
    match fields[3] {
        Element::MapField(f) => {
            assert_eq!(f.field.name, "scores");
            assert_eq!(f.key_type, "string");
            assert_eq!(f.field.type_name, "int32");
        }
        _ => panic!("expected MapField"),
    }
}

#[test]
fn test_parse_oneof() {
    let src = r#"syntax = "proto3";
message Sample {
    oneof test_oneof {
        string name = 1;
        int32 code = 2;
    }
}"#;
    let mut p = Parser::new(src);
    let def = p.parse().unwrap();
    let msg = match &def.elements[1] {
        Element::Message(m) => m,
        _ => panic!("expected Message"),
    };
    let oneof = match &msg.elements[0] {
        Element::Oneof(o) => o,
        _ => panic!("expected Oneof"),
    };
    assert_eq!(oneof.name, "test_oneof");
    assert_eq!(oneof.elements.len(), 2);
}

#[test]
fn test_parse_reserved() {
    let src = r#"message Foo {
    reserved 2, 15, 9 to 11;
    reserved "foo", "bar";
}"#;
    let mut p = Parser::new(src);
    let def = p.parse().unwrap();
    let msg = match &def.elements[0] {
        Element::Message(m) => m,
        _ => panic!("expected Message"),
    };
    // First reserved (numeric)
    match &msg.elements[0] {
        Element::Reserved(r) => {
            assert_eq!(r.ranges.len(), 3);
            assert_eq!(r.ranges[0].source_representation(), "2");
            assert_eq!(r.ranges[1].source_representation(), "15");
            assert_eq!(r.ranges[2].source_representation(), "9 to 11");
        }
        _ => panic!("expected Reserved"),
    }
    // Second reserved (names)
    match &msg.elements[1] {
        Element::Reserved(r) => {
            assert_eq!(r.field_names, vec!["foo", "bar"]);
        }
        _ => panic!("expected Reserved"),
    }
}

#[test]
fn test_walk() {
    let src = r#"syntax = "proto3";
package test;
message Foo {
    string name = 1;
}
message Bar {
    int32 id = 1;
}"#;
    let mut p = Parser::new(src);
    let def = p.parse().unwrap();

    use std::cell::RefCell;
    use std::rc::Rc;
    let message_names: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let names_clone = message_names.clone();
    let mut handlers: Vec<proto_parser::visitor::Handler> = vec![Box::new(move |e| {
        if let Element::Message(m) = e {
            names_clone.borrow_mut().push(m.name.clone());
        }
    })];
    proto_parser::visitor::walk(&def, &mut handlers);
    let names = message_names.borrow();
    assert_eq!(*names, vec!["Foo", "Bar"]);
}

#[test]
fn test_parse_option_aggregate() {
    let src = r#"syntax = "proto3";
message Foo {
    string bar = 1 [(validate.rules).string = {min_len: 1, max_len: 100}];
}"#;
    let mut p = Parser::new(src);
    let result = p.parse();
    assert!(result.is_ok(), "parse error: {:?}", result.err());
}

#[test]
fn test_parse_extensions() {
    let src = r#"message Foo {
    extensions 100 to 199;
}"#;
    let mut p = Parser::new(src);
    let def = p.parse().unwrap();
    let msg = match &def.elements[0] {
        Element::Message(m) => m,
        _ => panic!("expected Message"),
    };
    match &msg.elements[0] {
        Element::Extensions(e) => {
            assert_eq!(e.ranges.len(), 1);
            assert_eq!(e.ranges[0].from, 100);
            assert_eq!(e.ranges[0].to, 199);
        }
        _ => panic!("expected Extensions"),
    }
}
