use std::rc::Rc;

use qalc_analysis::Normalizer;
use qalc_ast::Spanned;
use qalc_ast::base_ast::{DeclarationKind, ExpressionKind};
use qalc_parser::parse_program;
use qalc_utils::{CompilerStep, Source};

fn normalize_expression(expr: &str) -> ExpressionKind<Spanned> {
  let code = format!("f :: num f = {};", expr);
  let source = Rc::new(Source::memory(code));
  let concrete = parse_program(&source).expect("parse");
  let normalized = Normalizer.transform(concrete, &mut vec![]).expect("normalize");
  match normalized.declarations.into_iter().next().unwrap().kind {
    DeclarationKind::Function { body, .. } => body.kind,
    _ => panic!("expected function declaration"),
  }
}

fn assert_identifier(kind: &ExpressionKind<Spanned>, expected: &str) {
  match kind {
    ExpressionKind::Identifier(name) => assert_eq!(name, expected, "identifier"),
    other => panic!("expected identifier {:?}, got {:?}", expected, other),
  }
}

fn assert_integer_literal(kind: &ExpressionKind<Spanned>, expected: i128) {
  match kind {
    ExpressionKind::IntegerLiteral { number, .. } => {
      assert_eq!(*number, expected, "integer literal")
    }
    other => panic!("expected integer literal {}, got {:?}", expected, other),
  }
}

fn assert_transform(kind: &ExpressionKind<Spanned>) -> (&ExpressionKind<Spanned>, &ExpressionKind<Spanned>) {
  match kind {
    ExpressionKind::Transform { callee, param } => (&callee.kind, &param.kind),
    other => panic!("expected Transform, got {:?}", other),
  }
}

#[test]
fn normalize_add_becomes_transform() {
  let kind = normalize_expression("1 + 2");
  let (callee, param) = assert_transform(&kind);
  assert_integer_literal(param, 2);
  let (op, left) = assert_transform(callee);
  assert_identifier(op, "+");
  assert_integer_literal(left, 1);
}

#[test]
fn normalize_subtract_becomes_transform() {
  let kind = normalize_expression("3 - 1");
  let (callee, param) = assert_transform(&kind);
  assert_integer_literal(param, 1);
  let (op, left) = assert_transform(callee);
  assert_identifier(op, "-");
  assert_integer_literal(left, 3);
}

#[test]
fn normalize_multiply_becomes_transform() {
  let kind = normalize_expression("2 * 3");
  let (callee, _) = assert_transform(&kind);
  let (op, _) = assert_transform(callee);
  assert_identifier(op, "*");
}

#[test]
fn normalize_pipe_flips_arguments() {
  // a |> f  ≡  f a  (el callee es f, no el operador pipe)
  let kind = normalize_expression("a |> f");
  let (callee, param) = assert_transform(&kind);
  assert_identifier(callee, "f");
  assert_identifier(param, "a");
}

#[test]
fn normalize_apply_becomes_transform() {
  // f $ x  ≡  f x
  let kind = normalize_expression("f $ x");
  let (callee, param) = assert_transform(&kind);
  assert_identifier(callee, "f");
  assert_identifier(param, "x");
}

#[test]
fn normalize_partial_left_becomes_lambda() {
  // (+ 1)  →  (a) -> Transform(Transform("+", a), 1)
  let kind = normalize_expression("(+ 1)");
  match kind {
    ExpressionKind::Lambda { params, body } => {
      assert_eq!(params, vec!["a".to_string()]);
      match body.kind {
        ExpressionKind::Transform { callee: outer_callee, param: outer_param } => {
          assert_integer_literal(&outer_param.kind, 1);
          match outer_callee.kind {
            ExpressionKind::Transform { callee: op, param: inner_param } => {
              assert_identifier(&op.kind, "+");
              assert_identifier(&inner_param.kind, "a");
            }
            other => panic!("expected inner Transform, got {:?}", other),
          }
        }
        other => panic!("expected body Transform, got {:?}", other),
      }
    }
    other => panic!("expected Lambda, got {:?}", other),
  }
}

#[test]
fn normalize_partial_right_becomes_transform() {
  // (1 +)  →  Transform(Identifier("+"), 1)
  let kind = normalize_expression("(1 +)");
  let (callee, param) = assert_transform(&kind);
  assert_identifier(callee, "+");
  assert_integer_literal(param, 1);
}
