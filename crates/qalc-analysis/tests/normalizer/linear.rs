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
  let normalized = Normalizer
    .transform(concrete, &mut vec![])
    .expect("normalize");
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

fn assert_transform<'a>(
  kind: &'a ExpressionKind<Spanned>,
) -> (&'a ExpressionKind<Spanned>, &'a ExpressionKind<Spanned>) {
  match kind {
    ExpressionKind::Transform { callee, param } => (&callee.kind, &param.kind),
    other => panic!("expected Transform, got {:?}", other),
  }
}

#[test]
fn normalize_linear_pure_assignment() {
  // linear { x = 1, return x }  →  LetBlock { strict: false, [(x, 1)], x }
  let kind = normalize_expression("linear { x = 1, return x }");
  match kind {
    ExpressionKind::LetBlock {
      strict,
      assigments,
      value,
    } => {
      assert!(!strict, "pure assignment must not be strict");
      assert_eq!(assigments.len(), 1);
      let (pattern, expr) = &assigments[0];
      match &pattern.kind {
        PatternKind::Variable(name) => assert_eq!(name, "x"),
        other => panic!("expected variable pattern, got {:?}", other),
      }
      assert_integer_literal(&expr.kind, 1);
      assert_identifier(&value.kind, "x");
    }
    other => panic!("expected LetBlock, got {:?}", other),
  }
}

#[test]
fn normalize_linear_no_return_value_is_unit() {
  // linear { x = 1 }  →  LetBlock { ..., value: Tuple([]) }
  let kind = normalize_expression("linear { x = 1 }");
  match kind {
    ExpressionKind::LetBlock { value, .. } => match &value.kind {
      ExpressionKind::Tuple(elems) => assert!(elems.is_empty(), "expected empty tuple"),
      other => panic!("expected empty tuple as value, got {:?}", other),
    },
    other => panic!("expected LetBlock, got {:?}", other),
  }
}

#[test]
fn normalize_linear_side_effect_becomes_letstrict() {
  // linear { sideExpr, return 1 }  →  letstrict _ = sideExpr in 1
  let kind = normalize_expression("linear { sideExpr, return 1 }");
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
        other => panic!("expected wildcard pattern, got {:?}", other),
      }
      assert_identifier(&assigments[0].1.kind, "sideExpr");
      assert_integer_literal(&value.kind, 1);
    }
    other => panic!("expected LetBlock for side effect, got {:?}", other),
  }
}

#[test]
fn normalize_linear_monadic_single_context() {
  // linear { result <- op ctx, return result }
  // → LetBlock { [(Tuple([ctx, result]), Transform(op, ctx))], result }
  let kind = normalize_expression("linear { result <- op ctx, return result }");
  match kind {
    ExpressionKind::LetBlock {
      strict,
      assigments,
      value,
    } => {
      assert!(!strict, "monadic assignment must not be strict");
      assert_eq!(assigments.len(), 1);
      let (pattern, expr) = &assigments[0];
      match &pattern.kind {
        PatternKind::Tuple(inner) => {
          assert_eq!(inner.len(), 2, "tuple must have ctx + result");
          match &inner[0].kind {
            PatternKind::Variable(name) => assert_eq!(name, "ctx"),
            other => panic!("expected ctx, got {:?}", other),
          }
          match &inner[1].kind {
            PatternKind::Variable(name) => assert_eq!(name, "result"),
            other => panic!("expected result, got {:?}", other),
          }
        }
        other => panic!("expected tuple pattern, got {:?}", other),
      }
      let (callee, param) = assert_transform(&expr.kind);
      assert_identifier(callee, "op");
      assert_identifier(param, "ctx");
      assert_identifier(&value.kind, "result");
    }
    other => panic!("expected LetBlock, got {:?}", other),
  }
}

#[test]
fn normalize_linear_monadic_tuple_context() {
  // linear { result <- op (ctx1, ctx2), return result }
  // → LetBlock { [(Tuple([ctx1, ctx2, result]), Transform(op, Tuple([ctx1, ctx2])))], result }
  let kind = normalize_expression("linear { result <- op (ctx1, ctx2), return result }");
  match kind {
    ExpressionKind::LetBlock {
      assigments, value, ..
    } => {
      assert_eq!(assigments.len(), 1);
      let (pattern, expr) = &assigments[0];
      match &pattern.kind {
        PatternKind::Tuple(inner) => {
          assert_eq!(inner.len(), 3, "tuple must have ctx1 + ctx2 + result");
          match &inner[0].kind {
            PatternKind::Variable(n) => assert_eq!(n, "ctx1"),
            other => panic!("expected ctx1, got {:?}", other),
          }
          match &inner[1].kind {
            PatternKind::Variable(n) => assert_eq!(n, "ctx2"),
            other => panic!("expected ctx2, got {:?}", other),
          }
          match &inner[2].kind {
            PatternKind::Variable(n) => assert_eq!(n, "result"),
            other => panic!("expected result, got {:?}", other),
          }
        }
        other => panic!("expected tuple pattern, got {:?}", other),
      }
      let (callee, arg) = assert_transform(&expr.kind);
      assert_identifier(callee, "op");
      match arg {
        ExpressionKind::Tuple(elems) => {
          assert_eq!(elems.len(), 2);
          assert_identifier(&elems[0].kind, "ctx1");
          assert_identifier(&elems[1].kind, "ctx2");
        }
        other => panic!("expected Tuple arg, got {:?}", other),
      }
      assert_identifier(&value.kind, "result");
    }
    other => panic!("expected LetBlock, got {:?}", other),
  }
}
