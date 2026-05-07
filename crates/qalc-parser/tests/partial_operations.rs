use std::rc::Rc;

use qalc_ast::concrete_ast::{BinaryOperatorKind, ConcreteDeclarationKind, ConcreteExpressionKind};
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

fn assert_identifier(kind: &ConcreteExpressionKind, expected: &str) {
  match kind {
    ConcreteExpressionKind::Identifier(value) => {
      assert_eq!(value, expected, "identifier value");
    }
    other => panic!("expected identifier, got {:?}", other),
  }
}

#[test]
fn parse_partial_op_left() {
  let kind = parse_expression_kind("(+ 1)");

  match kind {
    ConcreteExpressionKind::PartialLeftBinary { operator, right } => {
      assert_eq!(operator.kind, BinaryOperatorKind::Add, "operator");
      assert_integer_literal(&right.kind, 1);
    }
    other => panic!("expected partial left binary, got {:?}", other),
  }
}

#[test]
fn parse_partial_op_left_with_multiply() {
  let kind = parse_expression_kind("(* 2)");

  match kind {
    ConcreteExpressionKind::PartialLeftBinary { operator, right } => {
      assert_eq!(operator.kind, BinaryOperatorKind::Multiply, "operator");
      assert_integer_literal(&right.kind, 2);
    }
    other => panic!("expected partial left binary, got {:?}", other),
  }
}

#[test]
fn parse_partial_op_right() {
  let kind = parse_expression_kind("(1 +)");

  match kind {
    ConcreteExpressionKind::PartialRightBinary { left, operator } => {
      assert_eq!(operator.kind, BinaryOperatorKind::Add, "operator");
      assert_integer_literal(&left.kind, 1);
    }
    other => panic!("expected partial right binary, got {:?}", other),
  }
}

#[test]
fn parse_partial_op_right_with_raised() {
  let kind = parse_expression_kind("(2 **)");

  match kind {
    ConcreteExpressionKind::PartialRightBinary { left, operator } => {
      assert_eq!(operator.kind, BinaryOperatorKind::Raised, "operator");
      assert_integer_literal(&left.kind, 2);
    }
    other => panic!("expected partial right binary, got {:?}", other),
  }
}

#[test]
fn parse_partial_op_right_with_identifier() {
  let kind = parse_expression_kind("(x -)");

  match kind {
    ConcreteExpressionKind::PartialRightBinary { left, operator } => {
      assert_eq!(operator.kind, BinaryOperatorKind::Subtract, "operator");
      assert_identifier(&left.kind, "x");
    }
    other => panic!("expected partial right binary, got {:?}", other),
  }
}
