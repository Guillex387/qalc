use std::rc::Rc;

use qalc_ast::concrete_ast::{ConcreteDeclarationKind, ConcreteExpressionKind};
use qalc_parser::parse_program;
use qalc_utils::Source;

fn parse_identifier_kind(identifier: &str) -> ConcreteExpressionKind {
  let code = format!("id :: num id = {};", identifier);
  let source = Rc::new(Source::memory(code));
  let program = parse_program(&source).expect("parse program");

  let declaration = program
    .declarations
    .into_iter()
    .next()
    .expect("one declaration");

  match declaration.kind {
    ConcreteDeclarationKind::Function { body, .. } => body.kind,
    _ => panic!("expected function declaration"),
  }
}

#[test]
fn parse_simple_identifiers_test() {
  let cases: Vec<(&str, &str)> = vec![
    ("foo", "foo"),
    ("foo_bar1", "foo_bar1"),
    ("_leading", "_leading"),
  ];

  for (literal, expected) in cases {
    match parse_identifier_kind(literal) {
      ConcreteExpressionKind::Identifier(value) => {
        assert_eq!(value, expected, "identifier {}", literal);
      }
      other => panic!("expected identifier, got {:?}", other),
    }
  }
}

#[test]
fn parse_operator_identifiers_test() {
  let cases: Vec<(&str, &str)> = vec![("`+`", "+"), ("`||`", "||"), ("`**`", "**")];

  for (literal, expected) in cases {
    match parse_identifier_kind(literal) {
      ConcreteExpressionKind::Identifier(value) => {
        assert_eq!(value, expected, "identifier {}", literal);
      }
      other => panic!("expected identifier, got {:?}", other),
    }
  }
}

#[test]
fn reject_keyword_as_identifier_test() {
  let code = "id :: num id = if";
  let source = Rc::new(Source::memory(code));
  let result = parse_program(&source);

  assert!(result.is_err(), "expected keyword identifier to fail");
}
