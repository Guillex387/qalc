use qalc_utils::Span;

use crate::concrete_ast::{FloatSuffix, NumberSuffix, Pattern, SliceExpression, TypeExpression};

#[derive(Debug, Clone)]
pub struct NormalizedAST {
  pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub struct Declaration {
  pub kind: DeclarationKind,
  pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Expression {
  pub kind: ExpressionKind,
  pub span: Span,
}

#[derive(Debug, Clone)]
pub enum DeclarationKind {
  Type {
    identifier: String,
    attributes: Vec<(String, TypeExpression)>,
    linear: bool,
  },
  Function {
    identifier: String,
    type_annotation: TypeExpression,
    params: Vec<String>,
    body: Expression,
  },
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
  LetBlock {
    assigments: Vec<(Pattern, Expression)>,
    value: Box<Expression>,
  },
  IfElseBlock {
    test: Box<Expression>,
    consequent: Box<Expression>,
    alternative: Box<Expression>,
  },
  Call {
    callee: Box<Expression>,
    params: Vec<Expression>,
  },
  Lambda {
    params: Vec<String>,
    body: Box<Expression>,
  },
  Tuple(Vec<Expression>),
  Slice(SliceExpression),
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
