use std::rc::Rc;

use qalc_ast::concrete_ast::{DeclarationKind, ExpressionKind, NumberSuffix, SliceExpression};
use qalc_parser::parse_program;
use qalc_utils::Source;

fn parse_expression_kind(expression: &str) -> ExpressionKind {
  let code = format!("f :: num f = {}", expression);
  let source = Rc::new(Source::memory(code));
  let program = parse_program(&source).expect("parse program");

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

fn assert_integer_literal(kind: &ExpressionKind, expected: i128) {
  match kind {
    ExpressionKind::IntegerLiteral { number, suffix } => {
      assert_eq!(*number, expected, "integer literal value");
      assert_eq!(*suffix, NumberSuffix::Unknow, "integer literal suffix");
    }
    other => panic!("expected integer literal, got {:?}", other),
  }
}

#[test]
fn parse_empty_slice_items() {
  let kind = parse_expression_kind("[]");

  match kind {
    ExpressionKind::Slice(SliceExpression::Items(items)) => {
      assert!(items.is_empty(), "empty slice items");
    }
    other => panic!("expected slice items, got {:?}", other),
  }
}

#[test]
fn parse_slice_items() {
  let kind = parse_expression_kind("[1, 2, 3]");

  match kind {
    ExpressionKind::Slice(SliceExpression::Items(items)) => {
      assert_eq!(items.len(), 3, "slice items length");
      assert_integer_literal(&items[0].kind, 1);
      assert_integer_literal(&items[1].kind, 2);
      assert_integer_literal(&items[2].kind, 3);
    }
    other => panic!("expected slice items, got {:?}", other),
  }
}

#[test]
fn parse_nested_slice_items() {
  let kind = parse_expression_kind("[1, [2, 3]]");

  match kind {
    ExpressionKind::Slice(SliceExpression::Items(items)) => {
      assert_eq!(items.len(), 2, "slice items length");
      assert_integer_literal(&items[0].kind, 1);

      match &items[1].kind {
        ExpressionKind::Slice(SliceExpression::Items(nested_items)) => {
          assert_eq!(nested_items.len(), 2, "nested slice items length");
          assert_integer_literal(&nested_items[0].kind, 2);
          assert_integer_literal(&nested_items[1].kind, 3);
        }
        other => panic!("expected nested slice items, got {:?}", other),
      }
    }
    other => panic!("expected slice items, got {:?}", other),
  }
}

#[test]
fn parse_slice_range() {
  let kind = parse_expression_kind("[1..10]");

  match kind {
    ExpressionKind::Slice(SliceExpression::Range(start, end)) => {
      assert_integer_literal(&start.kind, 1);
      assert_integer_literal(&end.kind, 10);
    }
    other => panic!("expected slice range, got {:?}", other),
  }
}
