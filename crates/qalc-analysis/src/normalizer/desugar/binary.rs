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
  let operator_span = callee.span.clone();
  let second_arg = transform_expression(*right);

  NExprKind::Lambda {
    params: vec!["a".to_string()],
    body: Box::new(NExpr {
      kind: NExprKind::Transform {
        callee: Box::new(NExpr {
          kind: NExprKind::Transform {
            callee,
            param: Box::new(NExpr {
              kind: NExprKind::Identifier("a".to_string()),
              span: operator_span,
            }),
          },
          span: span.clone(),
        }),
        param: Box::new(second_arg),
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
  NExprKind::Transform {
    callee: Box::new(operator_to_identifier(operator)),
    param: Box::new(transform_expression(*left)),
  }
}

pub fn transform_binary(
  left: Box<CExpr>,
  operator: BinaryOperator,
  right: Box<CExpr>,
) -> NExprKind {
  if let BinaryOperatorKind::Pipe = operator.kind {
    return transform_binary_pipe(left, right);
  }
  if let BinaryOperatorKind::Apply = operator.kind {
    return transform_binary_apply(left, right);
  }

  NExprKind::Transform {
    callee: Box::new(NExpr {
      span: left.span.join(&operator.span),
      kind: NExprKind::Transform {
        callee: Box::new(operator_to_identifier(operator)),
        param: Box::new(transform_expression(*left)),
      },
    }),
    param: Box::new(transform_expression(*right)),
  }
}

fn transform_binary_pipe(left: Box<CExpr>, right: Box<CExpr>) -> NExprKind {
  NExprKind::Transform {
    callee: Box::new(transform_expression(*right)),
    param: Box::new(transform_expression(*left)),
  }
}

fn transform_binary_apply(left: Box<CExpr>, right: Box<CExpr>) -> NExprKind {
  NExprKind::Transform {
    callee: Box::new(transform_expression(*left)),
    param: Box::new(transform_expression(*right)),
  }
}
