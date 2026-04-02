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
  Unknow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FloatSuffix {
  F32,
  F64,
  Unknow,
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
