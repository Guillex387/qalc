use std::rc::Rc;

use qalc_analysis::{HMInferrer, Normalizer};
use qalc_ast::base_ast::DeclarationKind;
use qalc_ast::typed_ast::{Type, TypedAST};
use qalc_parser::parse_program;
use qalc_utils::{CompilerStep, Source};

fn infer_program(src: &str) -> TypedAST {
  let source = Rc::new(Source::memory(src.to_string()));
  let concrete = parse_program(&source).expect("parse");
  let normalized = Normalizer.transform(concrete, &mut vec![]).expect("normalize");
  HMInferrer::default()
    .transform(normalized, &mut vec![])
    .expect("infer")
}

fn decl_body_type(ast: &TypedAST, idx: usize) -> &Type {
  match &ast.declarations[idx].kind {
    DeclarationKind::Function { body, .. } => &body.meta.ttype,
    _ => panic!("expected function declaration"),
  }
}

fn decl_type(ast: &TypedAST, idx: usize) -> &Type {
  match &ast.declarations[idx].meta.ttype {
    t => t,
  }
}

fn builtin(id: &str) -> Type {
  Type::Builtin {
    id: id.to_string(),
    params: vec![],
    linear: false,
  }
}

#[test]
fn infer_integer_literal_i32() {
  let ast = infer_program("f :: i32 f = 1i32;");
  assert_eq!(*decl_body_type(&ast, 0), builtin("i32"));
}

#[test]
fn infer_integer_literal_u64() {
  let ast = infer_program("f :: u64 f = 1u64;");
  assert_eq!(*decl_body_type(&ast, 0), builtin("u64"));
}

#[test]
fn infer_integer_literal_no_suffix_defaults_to_i32() {
  let ast = infer_program("f :: i32 f = 1;");
  assert_eq!(*decl_body_type(&ast, 0), builtin("i32"));
}

#[test]
fn infer_float_literal_f64() {
  let ast = infer_program("f :: f64 f = 1.0f64;");
  assert_eq!(*decl_body_type(&ast, 0), builtin("f64"));
}

#[test]
fn infer_float_literal_f32() {
  let ast = infer_program("f :: f32 f = 1.0f32;");
  assert_eq!(*decl_body_type(&ast, 0), builtin("f32"));
}

#[test]
fn infer_identity_function() {
  // f :: i32 -> i32   f x = x
  // body is x (a parameter), type must unify to i32
  let ast = infer_program("f :: i32 -> i32 f x = x;");
  let ty = decl_type(&ast, 0);
  assert_eq!(
    *ty,
    Type::Arrow(Box::new(builtin("i32")), Box::new(builtin("i32")))
  );
}

#[test]
fn infer_tuple_literal() {
  let ast = infer_program("f :: (i32, i32) f = (1i32, 2i32);");
  assert_eq!(
    *decl_body_type(&ast, 0),
    Type::Tuple(vec![builtin("i32"), builtin("i32")])
  );
}

#[test]
fn infer_let_block() {
  // let binding: x gets type i32, body returns x
  let ast = infer_program("f :: i32 f = let x = 1i32 in x;");
  assert_eq!(*decl_body_type(&ast, 0), builtin("i32"));
}

#[test]
fn infer_if_else() {
  // Both branches must have the same type; x is the condition parameter typed as i32
  let ast = infer_program("f :: i32 -> i32 f x = if x then 1i32 else 2i32;");
  assert_eq!(*decl_body_type(&ast, 0), builtin("i32"));
}

#[test]
fn infer_lambda_type() {
  // f :: i32 -> i32   f = (x) -> x
  // The lambda body is an identifier that unifies with i32
  let ast = infer_program("f :: i32 -> i32 f = (x) -> x;");
  let ty = decl_type(&ast, 0);
  assert_eq!(
    *ty,
    Type::Arrow(Box::new(builtin("i32")), Box::new(builtin("i32")))
  );
}

#[test]
fn infer_two_declarations_independently() {
  let ast = infer_program("f :: i32 f = 1i32; g :: f64 g = 2.0f64;");
  assert_eq!(*decl_body_type(&ast, 0), builtin("i32"));
  assert_eq!(*decl_body_type(&ast, 1), builtin("f64"));
}
