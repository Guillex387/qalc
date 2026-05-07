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

fn assert_binary(
  kind: &ConcreteExpressionKind,
  expected: BinaryOperatorKind,
) -> (&ConcreteExpressionKind, &ConcreteExpressionKind) {
  match kind {
    ConcreteExpressionKind::Binary {
      left,
      operator,
      right,
    } => {
      assert_eq!(operator.kind, expected, "binary operator");
      (&left.kind, &right.kind)
    }
    other => panic!("expected binary expression, got {:?}", other),
  }
}

fn assert_binary_literals(
  kind: &ConcreteExpressionKind,
  expected: BinaryOperatorKind,
  left: i128,
  right: i128,
) {
  let (left_kind, right_kind) = assert_binary(kind, expected);
  assert_integer_literal(left_kind, left);
  assert_integer_literal(right_kind, right);
}

#[test]
fn parse_binary_operator_apply() {
  let kind = parse_expression_kind("1 $ 2");
  assert_binary_literals(&kind, BinaryOperatorKind::Apply, 1, 2);
}

#[test]
fn parse_binary_operator_pipe() {
  let kind = parse_expression_kind("1 |> 2");
  assert_binary_literals(&kind, BinaryOperatorKind::Pipe, 1, 2);
}

#[test]
fn parse_binary_operator_logic_or() {
  let kind = parse_expression_kind("1 || 2");
  assert_binary_literals(&kind, BinaryOperatorKind::LogicOr, 1, 2);
}

#[test]
fn parse_binary_operator_logic_and() {
  let kind = parse_expression_kind("1 && 2");
  assert_binary_literals(&kind, BinaryOperatorKind::LogicAnd, 1, 2);
}

#[test]
fn parse_binary_operator_or() {
  let kind = parse_expression_kind("1 | 2");
  assert_binary_literals(&kind, BinaryOperatorKind::Or, 1, 2);
}

#[test]
fn parse_binary_operator_and() {
  let kind = parse_expression_kind("1 & 2");
  assert_binary_literals(&kind, BinaryOperatorKind::And, 1, 2);
}

#[test]
fn parse_binary_operator_xor() {
  let kind = parse_expression_kind("1 ^ 2");
  assert_binary_literals(&kind, BinaryOperatorKind::Xor, 1, 2);
}

#[test]
fn parse_binary_operator_equal() {
  let kind = parse_expression_kind("1 == 2");
  assert_binary_literals(&kind, BinaryOperatorKind::Equal, 1, 2);
}

#[test]
fn parse_binary_operator_not_equal() {
  let kind = parse_expression_kind("1 != 2");
  assert_binary_literals(&kind, BinaryOperatorKind::NotEqual, 1, 2);
}

#[test]
fn parse_binary_operator_less() {
  let kind = parse_expression_kind("1 < 2");
  assert_binary_literals(&kind, BinaryOperatorKind::Less, 1, 2);
}

#[test]
fn parse_binary_operator_greater() {
  let kind = parse_expression_kind("1 > 2");
  assert_binary_literals(&kind, BinaryOperatorKind::Greater, 1, 2);
}

#[test]
fn parse_binary_operator_less_equal() {
  let kind = parse_expression_kind("1 <= 2");
  assert_binary_literals(&kind, BinaryOperatorKind::LessEqual, 1, 2);
}

#[test]
fn parse_binary_operator_greater_equal() {
  let kind = parse_expression_kind("1 >= 2");
  assert_binary_literals(&kind, BinaryOperatorKind::GreaterEqual, 1, 2);
}

#[test]
fn parse_binary_operator_add() {
  let kind = parse_expression_kind("1 + 2");
  assert_binary_literals(&kind, BinaryOperatorKind::Add, 1, 2);
}

#[test]
fn parse_binary_operator_subtract() {
  let kind = parse_expression_kind("1 - 2");
  assert_binary_literals(&kind, BinaryOperatorKind::Subtract, 1, 2);
}

#[test]
fn parse_binary_operator_multiply() {
  let kind = parse_expression_kind("1 * 2");
  assert_binary_literals(&kind, BinaryOperatorKind::Multiply, 1, 2);
}

#[test]
fn parse_binary_operator_divide() {
  let kind = parse_expression_kind("1 / 2");
  assert_binary_literals(&kind, BinaryOperatorKind::Divide, 1, 2);
}

#[test]
fn parse_binary_operator_module() {
  let kind = parse_expression_kind("1 % 2");
  assert_binary_literals(&kind, BinaryOperatorKind::Module, 1, 2);
}

#[test]
fn parse_binary_operator_raised() {
  let kind = parse_expression_kind("2 ** 3");
  assert_binary_literals(&kind, BinaryOperatorKind::Raised, 2, 3);
}

#[test]
fn parse_precedence_multiply_over_add() {
  let kind = parse_expression_kind("1 + 2 * 3");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperatorKind::Add);
  assert_integer_literal(left_kind, 1);
  let (mul_left, mul_right) = assert_binary(right_kind, BinaryOperatorKind::Multiply);
  assert_integer_literal(mul_left, 2);
  assert_integer_literal(mul_right, 3);
}

#[test]
fn parse_precedence_add_after_multiply() {
  let kind = parse_expression_kind("1 * 2 + 3");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperatorKind::Add);
  let (mul_left, mul_right) = assert_binary(left_kind, BinaryOperatorKind::Multiply);
  assert_integer_literal(mul_left, 1);
  assert_integer_literal(mul_right, 2);
  assert_integer_literal(right_kind, 3);
}

#[test]
fn parse_precedence_comparison_over_logic_and() {
  let kind = parse_expression_kind("1 < 2 && 3 < 4");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperatorKind::LogicAnd);
  let (left_left, left_right) = assert_binary(left_kind, BinaryOperatorKind::Less);
  assert_integer_literal(left_left, 1);
  assert_integer_literal(left_right, 2);
  let (right_left, right_right) = assert_binary(right_kind, BinaryOperatorKind::Less);
  assert_integer_literal(right_left, 3);
  assert_integer_literal(right_right, 4);
}

#[test]
fn parse_precedence_logic_over_bitwise() {
  let kind = parse_expression_kind("1 | 2 && 3");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperatorKind::LogicAnd);
  let (or_left, or_right) = assert_binary(left_kind, BinaryOperatorKind::Or);
  assert_integer_literal(or_left, 1);
  assert_integer_literal(or_right, 2);
  assert_integer_literal(right_kind, 3);
}

#[test]
fn parse_precedence_comparison_over_add() {
  let kind = parse_expression_kind("1 + 2 < 3 + 4");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperatorKind::Less);
  let (left_left, left_right) = assert_binary(left_kind, BinaryOperatorKind::Add);
  assert_integer_literal(left_left, 1);
  assert_integer_literal(left_right, 2);
  let (right_left, right_right) = assert_binary(right_kind, BinaryOperatorKind::Add);
  assert_integer_literal(right_left, 3);
  assert_integer_literal(right_right, 4);
}

#[test]
fn parse_precedence_pipe_over_apply() {
  let kind = parse_expression_kind("a |> b $ c");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperatorKind::Apply);
  let (pipe_left, pipe_right) = assert_binary(left_kind, BinaryOperatorKind::Pipe);
  assert_identifier(pipe_left, "a");
  assert_identifier(pipe_right, "b");
  assert_identifier(right_kind, "c");
}

#[test]
fn parse_associativity_left_subtract() {
  let kind = parse_expression_kind("10 - 3 - 2");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperatorKind::Subtract);
  let (inner_left, inner_right) = assert_binary(left_kind, BinaryOperatorKind::Subtract);
  assert_integer_literal(inner_left, 10);
  assert_integer_literal(inner_right, 3);
  assert_integer_literal(right_kind, 2);
}

#[test]
fn parse_associativity_left_divide() {
  let kind = parse_expression_kind("8 / 4 / 2");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperatorKind::Divide);
  let (inner_left, inner_right) = assert_binary(left_kind, BinaryOperatorKind::Divide);
  assert_integer_literal(inner_left, 8);
  assert_integer_literal(inner_right, 4);
  assert_integer_literal(right_kind, 2);
}

#[test]
fn parse_associativity_right_raised() {
  let kind = parse_expression_kind("2 ** 3 ** 2");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperatorKind::Raised);
  assert_integer_literal(left_kind, 2);
  let (inner_left, inner_right) = assert_binary(right_kind, BinaryOperatorKind::Raised);
  assert_integer_literal(inner_left, 3);
  assert_integer_literal(inner_right, 2);
}

#[test]
fn parse_associativity_right_apply() {
  let kind = parse_expression_kind("a $ b $ c");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperatorKind::Apply);
  assert_identifier(left_kind, "a");
  let (inner_left, inner_right) = assert_binary(right_kind, BinaryOperatorKind::Apply);
  assert_identifier(inner_left, "b");
  assert_identifier(inner_right, "c");
}

#[test]
fn parse_associativity_left_pipe() {
  let kind = parse_expression_kind("a |> b |> c");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperatorKind::Pipe);
  let (inner_left, inner_right) = assert_binary(left_kind, BinaryOperatorKind::Pipe);
  assert_identifier(inner_left, "a");
  assert_identifier(inner_right, "b");
  assert_identifier(right_kind, "c");
}

#[test]
fn parse_associativity_left_bitwise_group() {
  let kind = parse_expression_kind("1 | 2 ^ 3 & 4");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperatorKind::And);
  let (left_left, left_right) = assert_binary(left_kind, BinaryOperatorKind::Xor);
  let (or_left, or_right) = assert_binary(left_left, BinaryOperatorKind::Or);
  assert_integer_literal(or_left, 1);
  assert_integer_literal(or_right, 2);
  assert_integer_literal(left_right, 3);
  assert_integer_literal(right_kind, 4);
}

#[test]
fn parse_associativity_left_comparison_group() {
  let kind = parse_expression_kind("1 < 2 == 3");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperatorKind::Equal);
  let (inner_left, inner_right) = assert_binary(left_kind, BinaryOperatorKind::Less);
  assert_integer_literal(inner_left, 1);
  assert_integer_literal(inner_right, 2);
  assert_integer_literal(right_kind, 3);
}
