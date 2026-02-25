#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegerSuffix {
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
pub enum Expression {
  CallExpression { callee: Box<Expression>, params: Vec<Expression> },
  LambdaExpression { params: Vec<String>, body: Box<Expression> },
  Identifier(String),
  IntegerLiteral { number: String, suffix: IntegerSuffix },
}