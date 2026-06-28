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
fn normalize_single_arg_call() {
  // f 1  →  Transform(f, 1)
  let kind = normalize_expression("f 1");
  let (callee, param) = assert_transform(&kind);
  assert_identifier(callee, "f");
  assert_integer_literal(param, 1);
}

#[test]
fn normalize_multi_arg_call() {
  // f 1 2  →  Transform(Transform(f, 1), 2)
  let kind = normalize_expression("f 1 2");
  let (outer_callee, outer_param) = assert_transform(&kind);
  assert_integer_literal(outer_param, 2);
  let (fn_ident, first_arg) = assert_transform(outer_callee);
  assert_identifier(fn_ident, "f");
  assert_integer_literal(first_arg, 1);
}

#[test]
fn normalize_three_arg_call() {
  // f 1 2 3  →  Transform(Transform(Transform(f, 1), 2), 3)
  let kind = normalize_expression("f 1 2 3");
  let (lvl1_callee, lvl1_param) = assert_transform(&kind);
  assert_integer_literal(lvl1_param, 3);
  let (lvl2_callee, lvl2_param) = assert_transform(lvl1_callee);
  assert_integer_literal(lvl2_param, 2);
  let (fn_ident, first_arg) = assert_transform(lvl2_callee);
  assert_identifier(fn_ident, "f");
  assert_integer_literal(first_arg, 1);
}

#[test]
fn normalize_call_with_binary_arg() {
  // f (1 + 2)  →  Transform(f, Transform(Transform("+", 1), 2))
  let kind = normalize_expression("f (1 + 2)");
  let (callee, param) = assert_transform(&kind);
  assert_identifier(callee, "f");
  let (inner_callee, inner_param) = assert_transform(param);
  assert_integer_literal(inner_param, 2);
  let (op, left) = assert_transform(inner_callee);
  assert_identifier(op, "+");
  assert_integer_literal(left, 1);
}

#[test]
fn normalize_call_with_identifier_arg() {
  // f x  →  Transform(f, x)
  let kind = normalize_expression("f x");
  let (callee, param) = assert_transform(&kind);
  assert_identifier(callee, "f");
  assert_identifier(param, "x");
}
