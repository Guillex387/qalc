#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberSuffix {
  U8,
  U16,
  U32,
  U64,
  I8,
  I16,
  I32,
  I64,
  Unknow
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
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

#[derive(Debug, Clone)]
pub struct Span {
  pub start: usize,
  pub end: usize
}

impl Span {
  pub fn new(start: usize, end: usize) -> Self {
    Span { start, end }
  }
}

#[derive(Debug, Clone)]
pub struct Expression {
  pub kind: ExpressionKind,
  pub span: Span
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
  LetBlock { assigments: Vec<Expression>, value: Box<Expression> },
  IfElseBlock { test: Box<Expression>, consequent: Box<Expression>, alternative: Box<Expression> },
  MatchBlock { value: Box<Expression>, cases: Vec<(Expression, Expression)>, default: Option<Box<Expression>> },
  PerformBlock { sequence: Vec<Expression> },
  LinearBlock { expressions: Vec<Expression> },
  Binary { left: Box<Expression>, operator: BinaryOperator, right: Box<Expression> },
  PartialLeftBinary { operator: BinaryOperator, right: Box<Expression> },
  PartialRightBinary { left: Box<Expression>, operator: BinaryOperator },
  Call { callee: Box<Expression>, params: Vec<Expression> },
  Lambda { params: Vec<String>, body: Box<Expression> },
  Identifier(String),
  NumberLiteral { number: String, suffix: NumberSuffix },
  Assigment { identifier: String, value: Box<Expression> }
}