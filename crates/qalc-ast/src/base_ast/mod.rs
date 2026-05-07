mod const_enums;

use crate::ASTMetadata;
pub use const_enums::*;

#[derive(Debug, Clone)]
pub struct Declaration<M: ASTMetadata> {
  pub kind: DeclarationKind<M>,
  pub meta: M::DeclarationMeta,
}

#[derive(Debug, Clone)]
pub enum DeclarationKind<M: ASTMetadata> {
  Type {
    identifier: String,
    attributes: Vec<(String, TypeExpression<M>)>,
    linear: bool,
  },
  Function {
    identifier: String,
    type_annotation: TypeExpression<M>,
    params: Vec<String>,
    body: Expression<M>,
  },
}

#[derive(Debug, Clone)]
pub struct Pattern<M: ASTMetadata> {
  pub kind: PatternKind<M>,
  pub meta: M::PatternMeta,
}

#[derive(Debug, Clone)]
pub enum PatternKind<M: ASTMetadata> {
  Variable(String),
  Tuple(Vec<Pattern<M>>),
  Wildcard,
}

#[derive(Debug, Clone)]
pub struct TypeExpression<M: ASTMetadata> {
  pub kind: TypeExpressionKind<M>,
  pub meta: M::TypeMeta,
}

#[derive(Debug, Clone)]
pub enum TypeExpressionKind<M: ASTMetadata> {
  Identifier(String),
  IdentifierWithParams {
    identifier: String,
    params: Vec<TypeExpression<M>>,
  },
  Slice(Box<TypeExpression<M>>),
  Tuple(Vec<TypeExpression<M>>),
  Transform {
    input_type: Box<TypeExpression<M>>,
    output_type: Box<TypeExpression<M>>,
  },
}

#[derive(Debug, Clone)]
pub struct Expression<M: ASTMetadata> {
  pub kind: ExpressionKind<M>,
  pub meta: M::ExpressionMeta,
}

#[derive(Debug, Clone)]
pub enum ExpressionKind<M: ASTMetadata> {
  LetBlock {
    strict: bool,
    assigments: Vec<(Pattern<M>, Expression<M>)>,
    value: Box<Expression<M>>,
  },
  IfElseBlock {
    test: Box<Expression<M>>,
    consequent: Box<Expression<M>>,
    alternative: Box<Expression<M>>,
  },
  Transform {
    callee: Box<Expression<M>>,
    param: Box<Expression<M>>,
  },
  Lambda {
    params: Vec<String>,
    body: Box<Expression<M>>,
  },
  Tuple(Vec<Expression<M>>),
  Slice(SliceExpression<M>),
  Identifier(String),
  IntegerLiteral {
    number: i128,
    suffix: NumberSuffix,
  },
  FloatLiteral {
    number: f64,
    suffix: FloatSuffix,
  },
}

#[derive(Debug, Clone)]
pub enum SliceExpression<M: ASTMetadata> {
  Items(Vec<Expression<M>>),
  Range(Box<Expression<M>>, Box<Expression<M>>),
}
