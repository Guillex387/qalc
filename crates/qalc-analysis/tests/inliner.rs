use std::rc::Rc;

use qalc_analysis::{HMInferrer, Inliner, Normalizer, RecursionChecker};
use qalc_ast::base_ast::{Expression, ExpressionKind, SliceExpression};
use qalc_ast::flat_ast::{Flat, FlatAST, FlatMark};
use qalc_ast::typed_ast::{Type, TypeWithUnknow};
use qalc_builtins::{BuiltinDef, standard_builtins};
use qalc_parser::parse_program;
use qalc_utils::{CompilerResult, CompilerStep, Source};

/// Conduce el front-end completo sobre un fuente y devuelve el FlatAST.
fn flatten_with(src: &str, builtins: Vec<BuiltinDef>) -> CompilerResult<FlatAST> {
  let source = Rc::new(Source::memory(src.to_string()));
  let concrete = parse_program(&source).expect("parse");
  let mut diags = vec![];
  let normalized = Normalizer::default()
    .transform(concrete, &mut diags)
    .expect("normalize");
  let normalized = RecursionChecker::default()
    .transform(normalized, &mut diags)
    .expect("recursion");
  let typed = HMInferrer::new(builtins)
    .transform(normalized, &mut diags)
    .expect("infer");
  Inliner::default().transform(typed, &mut diags)
}

fn flatten(src: &str) -> CompilerResult<FlatAST> {
  flatten_with(src, standard_builtins())
}

fn i32_ty() -> TypeWithUnknow {
  TypeWithUnknow::Builtin {
    id: "i32".to_string(),
    params: vec![],
    linear: false,
  }
}

fn arrow(from: TypeWithUnknow, to: TypeWithUnknow) -> TypeWithUnknow {
  TypeWithUnknow::Arrow(Box::new(from), Box::new(to))
}

fn qio_of(inner: TypeWithUnknow) -> TypeWithUnknow {
  TypeWithUnknow::Builtin {
    id: "qio".to_string(),
    params: vec![inner],
    linear: false,
  }
}

/// Hijos directos (subexpresiones) de una expresión aplanada.
fn children(expr: &Expression<Flat>) -> Vec<&Expression<Flat>> {
  match &expr.kind {
    ExpressionKind::Transform { callee, param } => vec![callee.as_ref(), param.as_ref()],
    ExpressionKind::Lambda { body, .. } => vec![body.as_ref()],
    ExpressionKind::LetBlock {
      assigments, value, ..
    } => {
      let mut v: Vec<&Expression<Flat>> = assigments.iter().map(|(_, e)| e).collect();
      v.push(value.as_ref());
      v
    }
    ExpressionKind::IfElseBlock {
      test,
      consequent,
      alternative,
    } => vec![test.as_ref(), consequent.as_ref(), alternative.as_ref()],
    ExpressionKind::Tuple(exprs) => exprs.iter().collect(),
    ExpressionKind::Slice(slice) => match slice {
      SliceExpression::Items(items) => items.iter().collect(),
      SliceExpression::Range(start, end) => vec![start.as_ref(), end.as_ref()],
    },
    ExpressionKind::Identifier(_)
    | ExpressionKind::IntegerLiteral { .. }
    | ExpressionKind::FloatLiteral { .. } => vec![],
  }
}

fn idents(expr: &Expression<Flat>) -> Vec<String> {
  fn go(e: &Expression<Flat>, out: &mut Vec<String>) {
    if let ExpressionKind::Identifier(name) = &e.kind {
      out.push(name.clone());
    }
    for c in children(e) {
      go(c, out);
    }
  }
  let mut out = vec![];
  go(expr, &mut out);
  out
}

fn count_residual_lambdas(expr: &Expression<Flat>) -> usize {
  fn go(e: &Expression<Flat>, n: &mut usize) {
    if matches!(&e.kind, ExpressionKind::Lambda { .. }) && e.meta.mark == FlatMark::ResidualLambda {
      *n += 1;
    }
    for c in children(e) {
      go(c, n);
    }
  }
  let mut n = 0;
  go(expr, &mut n);
  n
}

#[test]
fn inlines_user_function_call() {
  let flat =
    flatten("g :: i32 -> i32  g x = x;  main :: i32 -> i32  main y = g y;").expect("flatten");

  // La llamada a `g` desaparece: queda un let (thunk) sin referencia a `g`.
  assert!(!idents(&flat.body).contains(&"g".to_string()));
  assert!(matches!(flat.body.kind, ExpressionKind::LetBlock { .. }));

  // El normalizador renombra los parámetros a `p0, p1, ...` (el nombre
  // original `y` pasa a un let de destructuring dentro del cuerpo).
  assert_eq!(flat.params.len(), 1);
  assert_eq!(flat.params[0].name, "p0");
  assert!(matches!(&flat.params[0].ttype, Type::Builtin { id, .. } if id == "i32"));

  assert_eq!(flat.result.arity, 1);
  assert!(!flat.result.qio);
}

#[test]
fn preserves_builtin_call() {
  let flat = flatten("main :: i32 -> i32  main y = add y y;").expect("flatten");

  // La llamada al builtin `add` se conserva: su identificador (que sólo
  // aparece como callee) sobrevive al aplanado.
  assert!(idents(&flat.body).contains(&"add".to_string()));
}

#[test]
fn inlines_nullary_function() {
  let flat = flatten("g :: i32  g = 5;  main :: i32  main = g;").expect("flatten");

  assert!(!idents(&flat.body).contains(&"g".to_string()));
  assert!(matches!(
    flat.body.kind,
    ExpressionKind::IntegerLiteral { .. }
  ));
}

#[test]
fn preserves_lambda_argument_to_builtin_and_inlines_inside() {
  // `hof` es un builtin de orden superior: su callback lambda debe conservarse.
  let mut builtins = standard_builtins();
  builtins.push(BuiltinDef::new(
    "hof",
    arrow(arrow(i32_ty(), i32_ty()), i32_ty()),
  ));

  let flat = flatten_with(
    "g :: i32 -> i32  g x = x;  main :: i32  main = hof ((z) -> g z);",
    builtins,
  )
  .expect("flatten");

  // Se conserva exactamente un lambda, marcado como residual.
  assert_eq!(count_residual_lambdas(&flat.body), 1);
  // El builtin de orden superior sigue ahí.
  assert!(idents(&flat.body).contains(&"hof".to_string()));
  // La llamada a `g` DENTRO del lambda se ha inlinado.
  assert!(!idents(&flat.body).contains(&"g".to_string()));
}

#[test]
fn inlines_lambda_passed_to_user_function() {
  // `apply` aplica su parámetro-función: la lambda pasada debe reducirse por
  // completo (nada de lambda "suelta"), no solo quedar ligada a un local.
  let src = "apply :: i32 -> (i32 -> i32) -> i32  apply x f = f x;  \
             main :: i32 -> i32  main n = apply n ((y) -> add y y);";
  let flat = flatten(src).expect("flatten");

  assert_eq!(count_residual_lambdas(&flat.body), 0);
  assert!(!idents(&flat.body).contains(&"apply".to_string()));
  // El builtin `add` (dentro de la lambda) se conserva.
  assert!(idents(&flat.body).contains(&"add".to_string()));
}

#[test]
fn result_simple_arity() {
  let flat = flatten("main :: i32  main = 5;").expect("flatten");
  assert_eq!(flat.result.arity, 1);
  assert!(!flat.result.qio);
  assert!(flat.params.is_empty());
}

#[test]
fn result_tuple_arity() {
  let flat = flatten("main :: (i32, i32)  main = (1, 2);").expect("flatten");
  assert_eq!(flat.result.arity, 2);
  assert!(!flat.result.qio);
}

#[test]
fn result_qio_wrapped() {
  let mut builtins = standard_builtins();
  builtins.push(BuiltinDef::new("mkqio", arrow(i32_ty(), qio_of(i32_ty()))));

  let flat = flatten_with("main :: qio<i32>  main = mkqio 5;", builtins).expect("flatten");
  assert!(flat.result.qio);
  assert_eq!(flat.result.arity, 1);
}

#[test]
fn partial_application_is_rejected() {
  let src = "g :: i32 -> i32 -> i32  g x y = add x y;  main :: i32 -> i32  main = g 1;";
  assert!(flatten(src).is_err());
}

#[test]
fn missing_main_is_rejected() {
  assert!(flatten("g :: i32  g = 5;").is_err());
}
