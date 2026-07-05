use std::rc::Rc;

use qalc_utils::Span;

use crate::{ASTMetadata, base_ast::Declaration, scope::Scope};

#[derive(Debug, Clone)]
pub struct InferenceMetadata {
  pub span: Span,
  pub ttype: TypeWithUnknow,
  pub scope: Rc<Scope>,
}

#[derive(Debug, Clone)]
pub struct Inferred;

impl ASTMetadata for Inferred {
  type DeclarationMeta = InferenceMetadata;
  type ExpressionMeta = InferenceMetadata;
  type PatternMeta = Span;
  type TypeMeta = Span;
}

/// Fuente de verdad única de qué tipos built-in son lineales. Los datos
/// cuánticos (un circuito o un dato cuántico) deben consumirse exactamente una
/// vez, mientras que `qio` es solo un envoltorio de efecto y no es lineal.
pub fn is_linear_builtin(id: &str) -> bool {
  matches!(id, "qcirc" | "qbool" | "qi16" | "qu8" | "qnum")
}

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

  /// Un tipo es clásico si no es lineal él mismo ni envuelve, directa o
  /// transitivamente (a través de tuplas, slices o el envoltorio de efecto
  /// `qio`), ningún tipo lineal. Es la condición que debe cumplir la firma de
  /// `main`, ya que solo los tipos clásicos pueden traducirse a los
  /// modificadores `input`/`output` de OpenQASM 3.
  pub fn is_classical(&self) -> bool {
    if self.is_linear() {
      return false;
    }
    match self {
      Type::Builtin { params, .. } => params.iter().all(Type::is_classical),
      Type::Arrow(_, _) => true,
      Type::Tuple(types) => types.iter().all(Type::is_classical),
      Type::Slice(ttype) => ttype.is_classical(),
    }
  }
}

impl TryFrom<TypeWithUnknow> for Type {
  type Error = String;

  fn try_from(value: TypeWithUnknow) -> Result<Self, Self::Error> {
    match value {
      TypeWithUnknow::Arrow(type_a, type_b) => {
        let type_a = Type::try_from(*type_a)?;
        let type_b = Type::try_from(*type_b)?;

        Ok(Type::Arrow(Box::new(type_a), Box::new(type_b)))
      }
      TypeWithUnknow::Builtin { id, params, linear } => {
        let params = params
          .into_iter()
          .map(Type::try_from)
          .collect::<Result<Vec<Type>, Self::Error>>()?;

        Ok(Type::Builtin { id, params, linear })
      }
      TypeWithUnknow::Slice(internal_type) => {
        let internal_type = Type::try_from(*internal_type)?;

        Ok(Type::Slice(Box::new(internal_type)))
      }
      TypeWithUnknow::Tuple(internal_types) => {
        let internal_types = internal_types
          .into_iter()
          .map(Type::try_from)
          .collect::<Result<Vec<Type>, Self::Error>>()?;

        Ok(Type::Tuple(internal_types))
      }
      TypeWithUnknow::Unknow(_) => Err("".to_string()),
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
