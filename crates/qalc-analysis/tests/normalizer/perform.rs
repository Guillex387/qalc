use std::rc::Rc;

use qalc_analysis::Normalizer;
use qalc_ast::Spanned;
use qalc_ast::base_ast::{DeclarationKind, ExpressionKind, PatternKind};
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

fn assert_transform<'a>(kind: &'a ExpressionKind<Spanned>) -> (&'a ExpressionKind<Spanned>, &'a ExpressionKind<Spanned>) {
  match kind {
    ExpressionKind::Transform { callee, param } => (&callee.kind, &param.kind),
    other => panic!("expected Transform, got {:?}", other),
  }
}

#[test]
fn normalize_perform_pure_assignment() {
  // perform { x = 1, return x }
  // → LetBlock { strict: false, [(x, 1)], monadPure(x) }
  let kind = normalize_expression("perform { x = 1, return x }");
  match kind {
    ExpressionKind::LetBlock {
      strict,
      assigments,
      ..
    } => {
      assert!(!strict, "pure assignment must not be strict");
      assert_eq!(assigments.len(), 1);
      let (pattern, expr) = &assigments[0];
      match &pattern.kind {
        PatternKind::Variable(name) => assert_eq!(name, "x"),
        other => panic!("expected variable pattern, got {:?}", other),
      }
      assert_integer_literal(&expr.kind, 1);
    }
    other => panic!("expected LetBlock, got {:?}", other),
  }
}

#[test]
fn normalize_perform_return_value_wrapped_in_monad_pure() {
  // perform { x = 1, return x }
  // The outer monad layer: the value of the whole block is monadPure(x)
  // which is represented as the continuation of nested LetBlock/Transform
  // We test that the innermost value is Transform(monadPure, x)
  let kind = normalize_expression("perform { x = 1, return x }");
  match kind {
    ExpressionKind::LetBlock { value, .. } => {
      let (callee, param) = assert_transform(&value.kind);
      assert_identifier(callee, "monadPure");
      assert_identifier(param, "x");
    }
    other => panic!("expected LetBlock, got {:?}", other),
  }
}

#[test]
fn normalize_perform_no_return_value_wraps_unit() {
  // perform { sideExpr }  →  letstrict _ = sideExpr in monadPure(())
  let kind = normalize_expression("perform { 1 }");
  match kind {
    ExpressionKind::LetBlock { strict, value, .. } => {
      assert!(strict, "side-effect expression must be strict");
      let (callee, param) = assert_transform(&value.kind);
      assert_identifier(callee, "monadPure");
      match &param {
        ExpressionKind::Tuple(elements) => assert!(elements.is_empty(), "void tuple"),
        other => panic!("expected empty tuple, got {:?}", other),
      }
    }
    other => panic!("expected LetBlock for side effect, got {:?}", other),
  }
}

#[test]
fn normalize_perform_side_effect_becomes_letstrict() {
  // perform { sideExpr, return 1 }  →  letstrict _ = sideExpr in monadPure(1)
  let kind = normalize_expression("perform { sideExpr, return 1 }");
  match kind {
    ExpressionKind::LetBlock {
      strict,
      assigments,
      value,
    } => {
      assert!(strict, "side-effect expression must produce a strict let");
      assert_eq!(assigments.len(), 1);
      match &assigments[0].0.kind {
        PatternKind::Wildcard => {}
        other => panic!("expected wildcard pattern for side effect, got {:?}", other),
      }
      assert_identifier(&assigments[0].1.kind, "sideExpr");
      let (callee, param) = assert_transform(&value.kind);
      assert_identifier(callee, "monadPure");
      assert_integer_literal(param, 1);
    }
    other => panic!("expected LetBlock for side effect, got {:?}", other),
  }
}

#[test]
fn normalize_perform_monadic_assignment() {
  // perform { result <- measure qbit, return result }
  // → Transform(Transform(monadMap, measure qbit), (a) -> let result = a in monadPure(result))
  let kind = normalize_expression("perform { result <- measure qbit, return result }");
  let (outer_callee, outer_param) = assert_transform(&kind);
  // outer_param must be the callback lambda
  match outer_param {
    ExpressionKind::Lambda { params, body } => {
      assert_eq!(params, &vec!["a".to_string()]);
      match &body.kind {
        ExpressionKind::LetBlock {
          strict,
          assigments,
          value,
        } => {
          assert!(!strict);
          assert_eq!(assigments.len(), 1);
          match &assigments[0].0.kind {
            PatternKind::Variable(name) => assert_eq!(name, "result"),
            other => panic!("expected 'result' pattern, got {:?}", other),
          }
          assert_identifier(&assigments[0].1.kind, "a");
          let (callee, ret_val) = assert_transform(&value.kind);
          assert_identifier(callee, "monadPure");
          assert_identifier(ret_val, "result");
        }
        other => panic!("expected LetBlock in lambda body, got {:?}", other),
      }
    }
    other => panic!("expected Lambda callback, got {:?}", other),
  }
  // outer_callee must be Transform(monadMap, measure qbit)
  let (monad_map, effect_call) = assert_transform(outer_callee);
  assert_identifier(monad_map, "monadMap");
  // effect_call is `measure qbit` → Transform(measure, qbit)
  let (measure, qbit) = assert_transform(effect_call);
  assert_identifier(measure, "measure");
  assert_identifier(qbit, "qbit");
}
