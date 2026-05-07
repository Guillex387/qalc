use std::rc::Rc;

use qalc_ast::Spanned;
use qalc_ast::base_ast::{Pattern, PatternKind};
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

fn assert_param_vars(params: &[Pattern<Spanned>], expected: &[&str]) {
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

fn assert_binary_add(kind: &ConcreteExpressionKind, left: i128, right: i128) {
  match kind {
    ConcreteExpressionKind::Binary {
      left: left_expr,
      operator,
      right: right_expr,
    } => {
      assert_eq!(operator.kind, BinaryOperatorKind::Add, "binary operator");
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
    ConcreteExpressionKind::Call { callee, params } => {
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
    ConcreteExpressionKind::Call { callee, params } => {
      match callee.kind {
        ConcreteExpressionKind::PartialLeftBinary { operator, right } => {
          assert_eq!(operator.kind, BinaryOperatorKind::Add, "callee operator");
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
    ConcreteExpressionKind::Call { callee, params } => {
      assert_identifier(&callee.kind, "f");
      assert_eq!(params.len(), 1, "params length");

      match &params[0].kind {
        ConcreteExpressionKind::Lambda { params, body } => {
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
    ConcreteExpressionKind::Call { callee, params } => {
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
    ConcreteExpressionKind::Call { callee, params } => {
      assert_identifier(&callee.kind, "f");
      assert_eq!(params.len(), 1, "params length");
      assert_identifier(&params[0].kind, "+");
    }
    other => panic!("expected call expression, got {:?}", other),
  }
}
