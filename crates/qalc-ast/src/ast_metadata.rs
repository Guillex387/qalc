use qalc_utils::Span;
use std::fmt::Debug;

pub trait ASTMetadata: Debug + Clone {
  type DeclarationMeta: Debug + Clone;
  type ExpressionMeta: Debug + Clone;
  type TypeMeta: Debug + Clone;
  type PatternMeta: Debug + Clone;
}

#[derive(Debug, Clone)]
pub struct Spanned;

impl ASTMetadata for Spanned {
  type DeclarationMeta = Span;
  type ExpressionMeta = Span;
  type TypeMeta = Span;
  type PatternMeta = Span;
}
