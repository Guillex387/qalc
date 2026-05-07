use std::rc::Rc;

use qalc_ast::base_ast::{FloatSuffix, NumberSuffix};
use qalc_ast::concrete_ast::{ConcreteDeclarationKind, ConcreteExpressionKind};
use qalc_parser::parse_program;
use qalc_utils::Source;

fn parse_literal_kind(literal: &str) -> ConcreteExpressionKind {
  let code = format!("lit :: num lit = {};", literal);
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

fn assert_float_close(actual: f64, expected: f64, literal: &str) {
  let delta = (actual - expected).abs();
  assert!(
    delta <= 1e-6,
    "literal {} expected {} got {}",
    literal,
    expected,
    actual
  );
}

#[test]
fn parse_integer_literals() {
  let cases: Vec<(&str, i128, NumberSuffix)> = vec![
    ("0", 0, NumberSuffix::Unknow),
    ("42", 42, NumberSuffix::Unknow),
    ("0x2a", 42, NumberSuffix::Unknow),
    ("0b1010", 10, NumberSuffix::Unknow),
    ("255u8", 255, NumberSuffix::U8),
    ("1024u16", 1024, NumberSuffix::U16),
    ("65536u32", 65536, NumberSuffix::U32),
    ("0x2au64", 42, NumberSuffix::U64),
    ("127i8", 127, NumberSuffix::I8),
    ("1i16", 1, NumberSuffix::I16),
    ("0b101i32", 5, NumberSuffix::I32),
    ("42i64", 42, NumberSuffix::I64),
  ];

  for (literal, expected, suffix) in cases {
    match parse_literal_kind(literal) {
      ConcreteExpressionKind::IntegerLiteral {
        number,
        suffix: parsed_suffix,
      } => {
        assert_eq!(number, expected, "literal {}", literal);
        assert_eq!(parsed_suffix, suffix, "literal {}", literal);
      }
      other => panic!("expected integer literal, got {:?}", other),
    }
  }
}

#[test]
fn parse_float_literals() {
  let cases: Vec<(&str, f64, FloatSuffix)> = vec![
    ("1.5", 1.5, FloatSuffix::Unknow),
    (".25", 0.25, FloatSuffix::Unknow),
    ("1e3", 1000.0, FloatSuffix::Unknow),
    ("1.5e2", 150.0, FloatSuffix::Unknow),
    ("2f32", 2.0, FloatSuffix::F32),
    ("3.25f32", 3.25, FloatSuffix::F32),
    (".5f32", 0.5, FloatSuffix::F32),
    ("2f64", 2.0, FloatSuffix::F64),
    ("3.25f64", 3.25, FloatSuffix::F64),
    (".5f64", 0.5, FloatSuffix::F64),
  ];

  for (literal, expected, suffix) in cases {
    match parse_literal_kind(literal) {
      ConcreteExpressionKind::FloatLiteral {
        number,
        suffix: parsed_suffix,
      } => {
        assert_float_close(number, expected, literal);
        assert_eq!(parsed_suffix, suffix, "literal {}", literal);
      }
      other => panic!("expected float literal, got {:?}", other),
    }
  }
}
