use std::rc::Rc;

use qalc_ast::concrete_ast::{
  BinaryOperator, DeclarationKind, ExpressionKind, Pattern, PatternKind,
};
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
    ExpressionKind::IntegerLiteral { number, .. } => {
      assert_eq!(*number, expected, "integer literal value");
    }
    other => panic!("expected integer literal, got {:?}", other),
  }
}

fn assert_identifier(kind: &ExpressionKind, expected: &str) {
  match kind {
    ExpressionKind::Identifier(value) => {
      assert_eq!(value, expected, "identifier value");
    }
    other => panic!("expected identifier, got {:?}", other),
  }
}

fn assert_param_vars(params: &[Pattern], expected: &[&str]) {
  assert_eq!(params.len(), expected.len(), "params length");

  for (param, expected_name) in params.iter().zip(expected.iter()) {
    match &param.kind {
      PatternKind::Variable(name) => {
        assert_eq!(name, expected_name, "param name");
      }
      other => panic!("expected variable pattern, got {:?}", other),
    }
  }
}

#[test]
fn parse_call_expression() {
  let kind = parse_expression_kind("f 1 2");

  match kind {
    ExpressionKind::Call { callee, params } => {
      assert_identifier(&callee.kind, "f");
      assert_eq!(params.len(), 2, "params length");
      assert_integer_literal(&params[0].kind, 1);
      assert_integer_literal(&params[1].kind, 2);
    }
    other => panic!("expected call expression, got {:?}", other),
  }
}

#[test]
fn parse_lambda_expression() {
  let kind = parse_expression_kind("(x y) => x");

  match kind {
    ExpressionKind::Lambda { params, body } => {
      assert_param_vars(&params, &["x", "y"]);
      assert_identifier(&body.kind, "x");
    }
    other => panic!("expected lambda expression, got {:?}", other),
  }
}

#[test]
fn parse_partial_op_left() {
  let kind = parse_expression_kind("(+ 1)");

  match kind {
    ExpressionKind::PartialLeftBinary { operator, right } => {
      assert_eq!(operator, BinaryOperator::Add, "operator");
      assert_integer_literal(&right.kind, 1);
    }
    other => panic!("expected partial left binary, got {:?}", other),
  }
}

#[test]
fn parse_partial_op_right() {
  let kind = parse_expression_kind("(1 +)");

  match kind {
    ExpressionKind::PartialRightBinary { left, operator } => {
      assert_eq!(operator, BinaryOperator::Add, "operator");
      assert_integer_literal(&left.kind, 1);
    }
    other => panic!("expected partial right binary, got {:?}", other),
  }
}
