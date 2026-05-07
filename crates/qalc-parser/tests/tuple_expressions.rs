use std::rc::Rc;

use qalc_ast::concrete_ast::{ConcreteDeclarationKind, ConcreteExpressionKind};
use qalc_parser::parse_program;
use qalc_utils::Source;

fn parse_expression_kind(expression: &str) -> ConcreteExpressionKind {
  let code = format!("f :: num f = {};", expression);
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

fn assert_integer_literal(kind: &ConcreteExpressionKind, expected: i128) {
  match kind {
    ConcreteExpressionKind::IntegerLiteral { number, .. } => {
      assert_eq!(*number, expected, "integer literal value");
    }
    other => panic!("expected integer literal, got {:?}", other),
  }
}

#[test]
fn parse_empty_tuple_expression() {
  let kind = parse_expression_kind("()");

  match kind {
    ConcreteExpressionKind::Tuple(items) => {
      assert!(items.is_empty(), "empty tuple");
    }
    other => panic!("expected tuple expression, got {:?}", other),
  }
}

#[test]
fn parse_tuple_expression_items() {
  let kind = parse_expression_kind("(1, 2, 3)");

  match kind {
    ConcreteExpressionKind::Tuple(items) => {
      assert_eq!(items.len(), 3, "tuple length");
      assert_integer_literal(&items[0].kind, 1);
      assert_integer_literal(&items[1].kind, 2);
      assert_integer_literal(&items[2].kind, 3);
    }
    other => panic!("expected tuple expression, got {:?}", other),
  }
}

#[test]
fn parse_nested_tuple_expression() {
  let kind = parse_expression_kind("(1, (2, 3))");

  match kind {
    ConcreteExpressionKind::Tuple(items) => {
      assert_eq!(items.len(), 2, "tuple length");
      assert_integer_literal(&items[0].kind, 1);

      match &items[1].kind {
        ConcreteExpressionKind::Tuple(nested_items) => {
          assert_eq!(nested_items.len(), 2, "nested tuple length");
          assert_integer_literal(&nested_items[0].kind, 2);
          assert_integer_literal(&nested_items[1].kind, 3);
        }
        other => panic!("expected nested tuple expression, got {:?}", other),
      }
    }
    other => panic!("expected tuple expression, got {:?}", other),
  }
}
