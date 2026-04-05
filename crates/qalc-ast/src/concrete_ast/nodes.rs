use qalc_utils::Span;

use crate::concrete_ast::enum_values::*;

#[derive(Debug, Clone)]
pub struct ConcreteAST {
  pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub struct Expression {
  pub kind: ExpressionKind,
  pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypeExpression {
  pub kind: TypeExpressionKind,
  pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Declaration {
  pub kind: DeclarationKind,
  pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Pattern {
  pub kind: PatternKind,
  pub span: Span,
}

#[derive(Debug, Clone)]
pub enum PatternKind {
  Variable(String),
  Tuple(Vec<Pattern>),
  Wildcard,
}

#[derive(Debug, Clone)]
pub enum SliceExpression {
  Items(Vec<Expression>),
  Range(Box<Expression>, Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
  LetBlock {
    assigments: Vec<Expression>,
    value: Box<Expression>,
  },
  IfElseBlock {
    test: Box<Expression>,
    consequent: Box<Expression>,
    alternative: Box<Expression>,
  },
  MatchBlock {
    value: Box<Expression>,
    cases: Vec<(Expression, Expression)>,
    default: Option<Box<Expression>>,
  },
  PerformBlock {
    sequence: Vec<Expression>,
  },
  LinearBlock {
    expressions: Vec<Expression>,
  },
  Binary {
    left: Box<Expression>,
    operator: BinaryOperator,
    right: Box<Expression>,
  },
  PartialLeftBinary {
    operator: BinaryOperator,
    right: Box<Expression>,
  },
  PartialRightBinary {
    left: Box<Expression>,
    operator: BinaryOperator,
  },
  Call {
    callee: Box<Expression>,
    params: Vec<Expression>,
  },
  Lambda {
    params: Vec<Pattern>,
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
  Assigment {
    identifier: Pattern,
    value: Box<Expression>,
  },
}

#[derive(Debug, Clone)]
pub enum TypeExpressionKind {
  Identifier(String),
  IdentifierWithParams {
    identifier: String,
    params: Vec<TypeExpression>,
  },
  Slice(Box<TypeExpression>),
  Tuple(Vec<TypeExpression>),
  Transform {
    input_type: Box<TypeExpression>,
    output_type: Box<TypeExpression>,
  },
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
    params: Vec<Pattern>,
    body: Expression,
  },
}
