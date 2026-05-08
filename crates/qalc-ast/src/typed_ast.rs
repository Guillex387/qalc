use qalc_utils::Span;

use crate::{ASTMetadata, base_ast::Declaration};

#[derive(Debug, Clone)]
pub enum Type {
  // TODO: add some flag for the linear restriction
  Builtin(String, Vec<Type>),
  Unknow(usize),
  Arrow(Box<Type>, Box<Type>),
  Tuple(Vec<Type>),
  Slice(Box<Type>),
}

#[derive(Debug, Clone)]
pub struct TypeMetadata {
  pub span: Span,
  pub ttype: Type,
  // TODO: pub scope: Rc<...>
}

#[derive(Debug, Clone)]
pub struct Typed;

impl ASTMetadata for Typed {
  type DeclarationMeta = TypeMetadata;
  type ExpressionMeta = TypeMetadata;
  type PatternMeta = Span;
  type TypeMeta = TypeMetadata;
}

#[derive(Debug, Clone)]
pub struct TypedAST {
  pub declarations: Vec<Declaration<Typed>>,
  // TODO: pub global_scope: Rc<...>
}
