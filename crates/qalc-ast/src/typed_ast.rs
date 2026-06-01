use std::rc::Rc;

use qalc_utils::Span;

use crate::{ASTMetadata, base_ast::Declaration, scope::Scope};

#[derive(Debug, Clone, PartialEq)]
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
      Type::Builtin {
        id: _,
        params: _,
        linear,
      } => *linear,
      Type::Arrow(_, _) => false,
      Type::Tuple(types) => types.iter().any(|ttype| ttype.is_linear()),
      Type::Slice(ttype) => ttype.is_linear(),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeWithUnknow {
  Builtin {
    id: String,
    params: Vec<TypeWithUnknow>,
    linear: bool,
  },
  Arrow(Box<TypeWithUnknow>, Box<TypeWithUnknow>),
  Tuple(Vec<TypeWithUnknow>),
  Slice(Box<TypeWithUnknow>),
  Unknow(usize),
}

impl TypeWithUnknow {
  pub fn is_linear(&self) -> bool {
    match self {
      TypeWithUnknow::Builtin { linear, .. } => *linear,
      TypeWithUnknow::Arrow(_, _) => false,
      TypeWithUnknow::Tuple(types) => types.iter().any(|t| t.is_linear()),
      TypeWithUnknow::Slice(t) => t.is_linear(),
      TypeWithUnknow::Unknow(_) => false,
    }
  }
}

impl From<Type> for TypeWithUnknow {
  fn from(t: Type) -> Self {
    match t {
      Type::Builtin { id, params, linear } => TypeWithUnknow::Builtin {
        id,
        params: params.into_iter().map(TypeWithUnknow::from).collect(),
        linear,
      },
      Type::Arrow(a, b) => TypeWithUnknow::Arrow(Box::new((*a).into()), Box::new((*b).into())),
      Type::Tuple(vec) => {
        TypeWithUnknow::Tuple(vec.into_iter().map(TypeWithUnknow::from).collect())
      }
      Type::Slice(boxed) => TypeWithUnknow::Slice(Box::new((*boxed).into())),
    }
  }
}

#[derive(Debug, Clone)]
pub struct TypeMetadata {
  pub span: Span,
  pub ttype: Type,
  pub scope: Rc<Scope>,
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
  pub global_scope: Rc<Scope>,
}
