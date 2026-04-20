use qalc_ast::concrete_ast::{BinaryOperator, BinaryOperatorKind, Expression as CExpr};
use qalc_ast::normalized_ast::{Expression as NExpr, ExpressionKind as NExprKind};
use qalc_utils::Span;

use crate::normalizer::expr::transform_expression;

fn operator_to_identifier(operator: BinaryOperator) -> NExpr {
  let ident = match &operator.kind {
    BinaryOperatorKind::LogicOr => "||",
    BinaryOperatorKind::LogicAnd => "&&",
    BinaryOperatorKind::Or => "|",
    BinaryOperatorKind::And => "&",
    BinaryOperatorKind::Xor => "^",
    BinaryOperatorKind::Equal => "==",
    BinaryOperatorKind::NotEqual => "!=",
    BinaryOperatorKind::LessEqual => "<=",
    BinaryOperatorKind::GreaterEqual => ">=",
    BinaryOperatorKind::Less => "<",
    BinaryOperatorKind::Greater => ">",
    BinaryOperatorKind::Add => "+",
    BinaryOperatorKind::Subtract => "-",
    BinaryOperatorKind::Multiply => "*",
    BinaryOperatorKind::Divide => "/",
    BinaryOperatorKind::Module => "%",
    BinaryOperatorKind::Raised => "**",
    _ => unreachable!("Expected overloadable operator, found {:?}", operator.kind),
  }
  .to_string();

  NExpr {
    kind: NExprKind::Identifier(ident),
    span: operator.span,
  }
}

// Example (source):
//   (+ 1)
// Example (normalized):
//   (a) -> (`+` a 1)
pub fn transform_partial_left(
  operator: BinaryOperator,
  right: Box<CExpr>,
  span: Span,
) -> NExprKind {
  let callee = Box::new(operator_to_identifier(operator));
  let second_arg = transform_expression(*right);

  NExprKind::Lambda {
    params: vec!["a".to_string()],
    body: Box::new(NExpr {
      kind: NExprKind::Call {
        callee,
        params: vec![
          NExpr {
            kind: NExprKind::Identifier("a".to_string()),
            span: span.clone(),
          },
          second_arg,
        ],
      },
      span: span.clone(),
    }),
  }
}

// Example (ssource):
//   (2 +)
// Example (normalized):
//   `+` 2
pub fn transform_partial_right(left: Box<CExpr>, operator: BinaryOperator) -> NExprKind {
  NExprKind::Call {
    callee: Box::new(operator_to_identifier(operator)),
    params: vec![transform_expression(*left)],
  }
}

pub fn transform_binary(
  left: Box<CExpr>,
  operator: BinaryOperator,
  right: Box<CExpr>,
) -> NExprKind {
  todo!()
}
