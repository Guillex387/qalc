use crate::{
  Spanned,
  base_ast::{FloatSuffix, NumberSuffix, Pattern, TypeExpression},
};
use qalc_utils::Span;

#[derive(Debug, Clone)]
pub struct ConcreteAST {
  pub declarations: Vec<ConcreteDeclaration>,
}

#[derive(Debug, Clone)]
pub struct ConcreteDeclaration {
  pub kind: ConcreteDeclarationKind,
  pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ConcreteDeclarationKind {
  Type {
    identifier: String,
    attributes: Vec<(String, TypeExpression<Spanned>)>,
    linear: bool,
  },
  Function {
    identifier: String,
    type_annotation: TypeExpression<Spanned>,
    params: Vec<Pattern<Spanned>>,
    body: ConcreteExpression,
  },
}

#[derive(Debug, Clone)]
pub struct ConcreteExpression {
  pub kind: ConcreteExpressionKind,
  pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ConcreteExpressionKind {
  LetBlock {
    assigments: Vec<ConcreteExpression>,
    value: Box<ConcreteExpression>,
  },
  IfElseBlock {
    test: Box<ConcreteExpression>,
    consequent: Box<ConcreteExpression>,
    alternative: Box<ConcreteExpression>,
  },
  MatchBlock {
    value: Box<ConcreteExpression>,
    cases: Vec<(ConcreteExpression, ConcreteExpression)>,
    default: Option<Box<ConcreteExpression>>,
  },
  PerformBlock {
    sequence: Vec<ConcreteExpression>,
    value: Option<Box<ConcreteExpression>>,
  },
  LinearBlock {
    sequence: Vec<ConcreteExpression>,
    value: Option<Box<ConcreteExpression>>,
  },
  Binary {
    left: Box<ConcreteExpression>,
    operator: BinaryOperator,
    right: Box<ConcreteExpression>,
  },
  PartialLeftBinary {
    operator: BinaryOperator,
    right: Box<ConcreteExpression>,
  },
  PartialRightBinary {
    left: Box<ConcreteExpression>,
    operator: BinaryOperator,
  },
  Call {
    callee: Box<ConcreteExpression>,
    params: Vec<ConcreteExpression>,
  },
  Lambda {
    params: Vec<Pattern<Spanned>>,
    body: Box<ConcreteExpression>,
  },
  Tuple(Vec<ConcreteExpression>),
  Slice(ConcreteSliceExpression),
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
    identifier: Pattern<Spanned>,
    value: Box<ConcreteExpression>,
    monadic: bool,
  },
}

#[derive(Debug, Clone)]
pub enum ConcreteSliceExpression {
  Items(Vec<ConcreteExpression>),
  Range(Box<ConcreteExpression>, Box<ConcreteExpression>),
}

#[derive(Debug, Clone)]
pub struct BinaryOperator {
  pub kind: BinaryOperatorKind,
  pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperatorKind {
  Apply,
  Pipe,
  LogicOr,
  LogicAnd,
  Or,
  And,
  Xor,
  Equal,
  NotEqual,
  LessEqual,
  GreaterEqual,
  Less,
  Greater,
  Add,
  Subtract,
  Multiply,
  Divide,
  Module,
  Raised,
}
