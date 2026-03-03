use crate::{ BinaryOperator, NumberSuffix, Span };

#[derive(Debug, Clone)]
pub struct Expression {
  pub kind: ExpressionKind,
  pub span: Span,
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
    params: Vec<String>,
    body: Box<Expression>,
  },
  Identifier(String),
  NumberLiteral {
    number: String,
    suffix: NumberSuffix,
  },
  Assigment {
    identifier: String,
    value: Box<Expression>,
  },
}
