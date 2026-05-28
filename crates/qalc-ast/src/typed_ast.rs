use qalc_utils::Span;

use crate::{ASTMetadata, base_ast::Declaration};

#[derive(Debug, Clone)]
pub enum Type {
  Builtin {
    id: String,
    params: Vec<Type>,
    linear: bool,
  },
  Arrow(Box<Type>, Box<Type>),
  Tuple(Vec<Type>),
  Slice(Box<Type>),
}

impl Type {
  pub fn is_linear(&self) -> bool {
    match self {
      Type::Builtin { id: _, params: _, linear } => *linear,
      Type::Arrow(_, _) => false,
      Type::Tuple(types) => types.iter().any(|ttype| ttype.is_linear()),
      Type::Slice(ttype) => ttype.is_linear(),
    }
  }
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
