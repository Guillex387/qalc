use std::rc::Rc;

use qalc_ast::concrete_ast::{BinaryOperator, DeclarationKind, ExpressionKind};
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

fn assert_binary(
  kind: &ExpressionKind,
  expected: BinaryOperator,
) -> (&ExpressionKind, &ExpressionKind) {
  match kind {
    ExpressionKind::Binary {
      left,
      operator,
      right,
    } => {
      assert_eq!(*operator, expected, "binary operator");
      (&left.kind, &right.kind)
    }
    other => panic!("expected binary expression, got {:?}", other),
  }
}

fn assert_binary_literals(
  kind: &ExpressionKind,
  expected: BinaryOperator,
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
  assert_binary_literals(&kind, BinaryOperator::Apply, 1, 2);
}

#[test]
fn parse_binary_operator_pipe() {
  let kind = parse_expression_kind("1 |> 2");
  assert_binary_literals(&kind, BinaryOperator::Pipe, 1, 2);
}

#[test]
fn parse_binary_operator_logic_or() {
  let kind = parse_expression_kind("1 || 2");
  assert_binary_literals(&kind, BinaryOperator::LogicOr, 1, 2);
}

#[test]
fn parse_binary_operator_logic_and() {
  let kind = parse_expression_kind("1 && 2");
  assert_binary_literals(&kind, BinaryOperator::LogicAnd, 1, 2);
}

#[test]
fn parse_binary_operator_or() {
  let kind = parse_expression_kind("1 | 2");
  assert_binary_literals(&kind, BinaryOperator::Or, 1, 2);
}

#[test]
fn parse_binary_operator_and() {
  let kind = parse_expression_kind("1 & 2");
  assert_binary_literals(&kind, BinaryOperator::And, 1, 2);
}

#[test]
fn parse_binary_operator_xor() {
  let kind = parse_expression_kind("1 ^ 2");
  assert_binary_literals(&kind, BinaryOperator::Xor, 1, 2);
}

#[test]
fn parse_binary_operator_equal() {
  let kind = parse_expression_kind("1 == 2");
  assert_binary_literals(&kind, BinaryOperator::Equal, 1, 2);
}

#[test]
fn parse_binary_operator_not_equal() {
  let kind = parse_expression_kind("1 != 2");
  assert_binary_literals(&kind, BinaryOperator::NotEqual, 1, 2);
}

#[test]
fn parse_binary_operator_less() {
  let kind = parse_expression_kind("1 < 2");
  assert_binary_literals(&kind, BinaryOperator::Less, 1, 2);
}

#[test]
fn parse_binary_operator_greater() {
  let kind = parse_expression_kind("1 > 2");
  assert_binary_literals(&kind, BinaryOperator::Greater, 1, 2);
}

#[test]
fn parse_binary_operator_less_equal() {
  let kind = parse_expression_kind("1 <= 2");
  assert_binary_literals(&kind, BinaryOperator::LessEqual, 1, 2);
}

#[test]
fn parse_binary_operator_greater_equal() {
  let kind = parse_expression_kind("1 >= 2");
  assert_binary_literals(&kind, BinaryOperator::GreaterEqual, 1, 2);
}

#[test]
fn parse_binary_operator_add() {
  let kind = parse_expression_kind("1 + 2");
  assert_binary_literals(&kind, BinaryOperator::Add, 1, 2);
}

#[test]
fn parse_binary_operator_subtract() {
  let kind = parse_expression_kind("1 - 2");
  assert_binary_literals(&kind, BinaryOperator::Subtract, 1, 2);
}

#[test]
fn parse_binary_operator_multiply() {
  let kind = parse_expression_kind("1 * 2");
  assert_binary_literals(&kind, BinaryOperator::Multiply, 1, 2);
}

#[test]
fn parse_binary_operator_divide() {
  let kind = parse_expression_kind("1 / 2");
  assert_binary_literals(&kind, BinaryOperator::Divide, 1, 2);
}

#[test]
fn parse_binary_operator_module() {
  let kind = parse_expression_kind("1 % 2");
  assert_binary_literals(&kind, BinaryOperator::Module, 1, 2);
}

#[test]
fn parse_binary_operator_raised() {
  let kind = parse_expression_kind("2 ** 3");
  assert_binary_literals(&kind, BinaryOperator::Raised, 2, 3);
}

#[test]
fn parse_precedence_multiply_over_add() {
  let kind = parse_expression_kind("1 + 2 * 3");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperator::Add);
  assert_integer_literal(left_kind, 1);
  let (mul_left, mul_right) = assert_binary(right_kind, BinaryOperator::Multiply);
  assert_integer_literal(mul_left, 2);
  assert_integer_literal(mul_right, 3);
}

#[test]
fn parse_precedence_add_after_multiply() {
  let kind = parse_expression_kind("1 * 2 + 3");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperator::Add);
  let (mul_left, mul_right) = assert_binary(left_kind, BinaryOperator::Multiply);
  assert_integer_literal(mul_left, 1);
  assert_integer_literal(mul_right, 2);
  assert_integer_literal(right_kind, 3);
}

#[test]
fn parse_precedence_comparison_over_logic_and() {
  let kind = parse_expression_kind("1 < 2 && 3 < 4");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperator::LogicAnd);
  let (left_left, left_right) = assert_binary(left_kind, BinaryOperator::Less);
  assert_integer_literal(left_left, 1);
  assert_integer_literal(left_right, 2);
  let (right_left, right_right) = assert_binary(right_kind, BinaryOperator::Less);
  assert_integer_literal(right_left, 3);
  assert_integer_literal(right_right, 4);
}

#[test]
fn parse_precedence_logic_over_bitwise() {
  let kind = parse_expression_kind("1 | 2 && 3");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperator::LogicAnd);
  let (or_left, or_right) = assert_binary(left_kind, BinaryOperator::Or);
  assert_integer_literal(or_left, 1);
  assert_integer_literal(or_right, 2);
  assert_integer_literal(right_kind, 3);
}

#[test]
fn parse_precedence_comparison_over_add() {
  let kind = parse_expression_kind("1 + 2 < 3 + 4");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperator::Less);
  let (left_left, left_right) = assert_binary(left_kind, BinaryOperator::Add);
  assert_integer_literal(left_left, 1);
  assert_integer_literal(left_right, 2);
  let (right_left, right_right) = assert_binary(right_kind, BinaryOperator::Add);
  assert_integer_literal(right_left, 3);
  assert_integer_literal(right_right, 4);
}

#[test]
fn parse_precedence_pipe_over_apply() {
  let kind = parse_expression_kind("a |> b $ c");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperator::Apply);
  let (pipe_left, pipe_right) = assert_binary(left_kind, BinaryOperator::Pipe);
  assert_identifier(pipe_left, "a");
  assert_identifier(pipe_right, "b");
  assert_identifier(right_kind, "c");
}

#[test]
fn parse_associativity_left_subtract() {
  let kind = parse_expression_kind("10 - 3 - 2");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperator::Subtract);
  let (inner_left, inner_right) = assert_binary(left_kind, BinaryOperator::Subtract);
  assert_integer_literal(inner_left, 10);
  assert_integer_literal(inner_right, 3);
  assert_integer_literal(right_kind, 2);
}

#[test]
fn parse_associativity_left_divide() {
  let kind = parse_expression_kind("8 / 4 / 2");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperator::Divide);
  let (inner_left, inner_right) = assert_binary(left_kind, BinaryOperator::Divide);
  assert_integer_literal(inner_left, 8);
  assert_integer_literal(inner_right, 4);
  assert_integer_literal(right_kind, 2);
}

#[test]
fn parse_associativity_right_raised() {
  let kind = parse_expression_kind("2 ** 3 ** 2");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperator::Raised);
  assert_integer_literal(left_kind, 2);
  let (inner_left, inner_right) = assert_binary(right_kind, BinaryOperator::Raised);
  assert_integer_literal(inner_left, 3);
  assert_integer_literal(inner_right, 2);
}

#[test]
fn parse_associativity_right_apply() {
  let kind = parse_expression_kind("a $ b $ c");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperator::Apply);
  assert_identifier(left_kind, "a");
  let (inner_left, inner_right) = assert_binary(right_kind, BinaryOperator::Apply);
  assert_identifier(inner_left, "b");
  assert_identifier(inner_right, "c");
}

#[test]
fn parse_associativity_left_pipe() {
  let kind = parse_expression_kind("a |> b |> c");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperator::Pipe);
  let (inner_left, inner_right) = assert_binary(left_kind, BinaryOperator::Pipe);
  assert_identifier(inner_left, "a");
  assert_identifier(inner_right, "b");
  assert_identifier(right_kind, "c");
}

#[test]
fn parse_associativity_left_bitwise_group() {
  let kind = parse_expression_kind("1 | 2 ^ 3 & 4");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperator::And);
  let (left_left, left_right) = assert_binary(left_kind, BinaryOperator::Xor);
  let (or_left, or_right) = assert_binary(left_left, BinaryOperator::Or);
  assert_integer_literal(or_left, 1);
  assert_integer_literal(or_right, 2);
  assert_integer_literal(left_right, 3);
  assert_integer_literal(right_kind, 4);
}

#[test]
fn parse_associativity_left_comparison_group() {
  let kind = parse_expression_kind("1 < 2 == 3");
  let (left_kind, right_kind) = assert_binary(&kind, BinaryOperator::Equal);
  let (inner_left, inner_right) = assert_binary(left_kind, BinaryOperator::Less);
  assert_integer_literal(inner_left, 1);
  assert_integer_literal(inner_right, 2);
  assert_integer_literal(right_kind, 3);
}
