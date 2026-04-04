use std::env;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

use qalc_ast::concrete_ast::{DeclarationKind, ExpressionKind};
use qalc_parser::parse_program;

static FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn write_temp_source(source: &str) -> std::path::PathBuf {
  let id = FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
  let filename = format!("qalc_identifier_test_{}_{}.qlc", std::process::id(), id);
  let path = env::temp_dir().join(filename);
  fs::write(&path, source).expect("write temp qalc source file");
  path
}

fn parse_identifier_kind(identifier: &str) -> ExpressionKind {
  let source = format!("id :: num id = {}", identifier);
  let path = write_temp_source(&source);
  let program = parse_program(path.to_str().expect("temp path to str")).expect("parse program");
  let _ = fs::remove_file(&path);

  let declaration = program
    .declarations
    .into_iter()
    .next()
    .expect("one declaration");

  match declaration.kind {
    DeclarationKind::Function { body, .. } => body.kind,
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
      ExpressionKind::Identifier(value) => {
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
      ExpressionKind::Identifier(value) => {
        assert_eq!(value, expected, "identifier {}", literal);
      }
      other => panic!("expected identifier, got {:?}", other),
    }
  }
}

#[test]
fn reject_keyword_as_identifier_test() {
  let source = "id :: num id = if";
  let path = write_temp_source(source);
  let result = parse_program(path.to_str().expect("temp path to str"));
  let _ = fs::remove_file(&path);

  assert!(result.is_err(), "expected keyword identifier to fail");
}
