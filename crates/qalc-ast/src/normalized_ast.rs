use qalc_utils::Span;

use crate::concrete_ast::{FloatSuffix, NumberSuffix, Pattern, TypeExpression};

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
pub enum SliceExpression {
  Items(Vec<Expression>),
  Range(Box<Expression>, Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
  LetBlock {
    strict: bool,
    assigments: Vec<(Pattern, Expression)>,
    value: Box<Expression>,
  },
  IfElseBlock {
    test: Box<Expression>,
    consequent: Box<Expression>,
    alternative: Box<Expression>,
  },
  Transform {
    callee: Box<Expression>,
    param: Box<Expression>,
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
