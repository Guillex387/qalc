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

#[derive(Debug)]
enum PatternShape {
  Variable(String),
  Tuple(Vec<PatternShape>),
  Wildcard,
}

fn assert_pattern_eq(actual: &Pattern, expected: &PatternShape) {
  match (&actual.kind, expected) {
    (PatternKind::Wildcard, PatternShape::Wildcard) => {}
    (PatternKind::Variable(actual_name), PatternShape::Variable(expected_name)) => {
      assert_eq!(actual_name, expected_name, "pattern variable");
    }
    (PatternKind::Tuple(actual_items), PatternShape::Tuple(expected_items)) => {
      assert_eq!(
        actual_items.len(),
        expected_items.len(),
        "tuple pattern length"
      );

      for (actual_item, expected_item) in actual_items.iter().zip(expected_items.iter()) {
        assert_pattern_eq(actual_item, expected_item);
      }
    }
    _ => panic!("expected {:?}, got {:?}", expected, actual.kind),
  }
}

fn assert_binary_add(kind: &ExpressionKind, left: i128, right: i128) {
  match kind {
    ExpressionKind::Binary {
      left: left_expr,
      operator,
      right: right_expr,
    } => {
      assert_eq!(*operator, BinaryOperator::Add, "binary operator");
      assert_integer_literal(&left_expr.kind, left);
      assert_integer_literal(&right_expr.kind, right);
    }
    other => panic!("expected binary expression, got {:?}", other),
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
fn parse_call_expression_with_partial_op_callee() {
  let kind = parse_expression_kind("((+ 1)) 22");

  match kind {
    ExpressionKind::Call { callee, params } => {
      match callee.kind {
        ExpressionKind::PartialLeftBinary { operator, right } => {
          assert_eq!(operator, BinaryOperator::Add, "callee operator");
          assert_integer_literal(&right.kind, 1);
        }
        other => panic!("expected partial left binary callee, got {:?}", other),
      }

      assert_eq!(params.len(), 1, "params length");
      assert_integer_literal(&params[0].kind, 22);
    }
    other => panic!("expected call expression, got {:?}", other),
  }
}

#[test]
fn parse_call_expression_with_lambda_param() {
  let kind = parse_expression_kind("f (x) -> x");

  match kind {
    ExpressionKind::Call { callee, params } => {
      assert_identifier(&callee.kind, "f");
      assert_eq!(params.len(), 1, "params length");

      match &params[0].kind {
        ExpressionKind::Lambda { params, body } => {
          assert_param_vars(params, &["x"]);
          assert_identifier(&body.kind, "x");
        }
        other => panic!("expected lambda param, got {:?}", other),
      }
    }
    other => panic!("expected call expression, got {:?}", other),
  }
}

#[test]
fn parse_call_expression_with_parenthesized_param() {
  let kind = parse_expression_kind("f (1 + 2)");

  match kind {
    ExpressionKind::Call { callee, params } => {
      assert_identifier(&callee.kind, "f");
      assert_eq!(params.len(), 1, "params length");
      assert_binary_add(&params[0].kind, 1, 2);
    }
    other => panic!("expected call expression, got {:?}", other),
  }
}

#[test]
fn parse_call_expression_with_operator_identifier_param() {
  let kind = parse_expression_kind("f `+`");

  match kind {
    ExpressionKind::Call { callee, params } => {
      assert_identifier(&callee.kind, "f");
      assert_eq!(params.len(), 1, "params length");
      assert_identifier(&params[0].kind, "+");
    }
    other => panic!("expected call expression, got {:?}", other),
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
fn parse_partial_op_left_with_multiply() {
  let kind = parse_expression_kind("(* 2)");

  match kind {
    ExpressionKind::PartialLeftBinary { operator, right } => {
      assert_eq!(operator, BinaryOperator::Multiply, "operator");
      assert_integer_literal(&right.kind, 2);
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

#[test]
fn parse_partial_op_right_with_raised() {
  let kind = parse_expression_kind("(2 **)");

  match kind {
    ExpressionKind::PartialRightBinary { left, operator } => {
      assert_eq!(operator, BinaryOperator::Raised, "operator");
      assert_integer_literal(&left.kind, 2);
    }
    other => panic!("expected partial right binary, got {:?}", other),
  }
}

#[test]
fn parse_partial_op_right_with_identifier() {
  let kind = parse_expression_kind("(x -)");

  match kind {
    ExpressionKind::PartialRightBinary { left, operator } => {
      assert_eq!(operator, BinaryOperator::Subtract, "operator");
      assert_identifier(&left.kind, "x");
    }
    other => panic!("expected partial right binary, got {:?}", other),
  }
}

#[test]
fn parse_lambda_expression() {
  let kind = parse_expression_kind("(x y) -> x");

  match kind {
    ExpressionKind::Lambda { params, body } => {
      assert_param_vars(&params, &["x", "y"]);
      assert_identifier(&body.kind, "x");
    }
    other => panic!("expected lambda expression, got {:?}", other),
  }
}

#[test]
fn parse_lambda_expression_with_wildcard_param() {
  let kind = parse_expression_kind("(_ x) -> x");

  match kind {
    ExpressionKind::Lambda { params, body } => {
      assert_eq!(params.len(), 2, "params length");
      assert_pattern_eq(&params[0], &PatternShape::Wildcard);
      assert_pattern_eq(&params[1], &PatternShape::Variable("x".to_string()));
      assert_identifier(&body.kind, "x");
    }
    other => panic!("expected lambda expression, got {:?}", other),
  }
}

#[test]
fn parse_lambda_expression_with_tuple_param() {
  let kind = parse_expression_kind("((a, _)) -> a");

  let expected = PatternShape::Tuple(vec![
    PatternShape::Variable("a".to_string()),
    PatternShape::Wildcard,
  ]);

  match kind {
    ExpressionKind::Lambda { params, body } => {
      assert_eq!(params.len(), 1, "params length");
      assert_pattern_eq(&params[0], &expected);
      assert_identifier(&body.kind, "a");
    }
    other => panic!("expected lambda expression, got {:?}", other),
  }
}
