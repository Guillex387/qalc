use qalc_ast::Spanned;
use qalc_ast::base_ast::{Expression, ExpressionKind};
use qalc_ast::concrete_ast::{BinaryOperator, BinaryOperatorKind, ConcreteExpression};
use qalc_utils::{CompilerResult, Span};

use crate::normalizer::expr::transform_expression;

fn operator_to_identifier(operator: BinaryOperator) -> Expression<Spanned> {
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

  Expression {
    kind: ExpressionKind::Identifier(ident),
    meta: operator.span,
  }
}

// Example (source):
//   (+ 1)
// Example (normalized):
//   (a) -> (`+` a 1)
pub fn transform_partial_left(
  operator: BinaryOperator,
  right: Box<ConcreteExpression>,
  span: Span,
) -> CompilerResult<ExpressionKind<Spanned>> {
  let callee = Box::new(operator_to_identifier(operator));
  let operator_span = callee.meta.clone();
  let second_arg = transform_expression(*right)?;

  Ok(ExpressionKind::Lambda {
    params: vec!["a".to_string()],
    body: Box::new(Expression {
      kind: ExpressionKind::Transform {
        callee: Box::new(Expression {
          kind: ExpressionKind::Transform {
            callee,
            param: Box::new(Expression {
              kind: ExpressionKind::Identifier("a".to_string()),
              meta: operator_span,
            }),
          },
          meta: span.clone(),
        }),
        param: Box::new(second_arg),
      },
      meta: span.clone(),
    }),
  })
}

// Example (ssource):
//   (2 +)
// Example (normalized):
//   `+` 2
pub fn transform_partial_right(
  left: Box<ConcreteExpression>,
  operator: BinaryOperator,
) -> CompilerResult<ExpressionKind<Spanned>> {
  Ok(ExpressionKind::Transform {
    callee: Box::new(operator_to_identifier(operator)),
    param: Box::new(transform_expression(*left)?),
  })
}

pub fn transform_binary(
  left: Box<ConcreteExpression>,
  operator: BinaryOperator,
  right: Box<ConcreteExpression>,
) -> CompilerResult<ExpressionKind<Spanned>> {
  if let BinaryOperatorKind::Pipe = operator.kind {
    return transform_binary_pipe(left, right);
  }
  if let BinaryOperatorKind::Apply = operator.kind {
    return transform_binary_apply(left, right);
  }

  Ok(ExpressionKind::Transform {
    callee: Box::new(Expression {
      meta: left.span.join(&operator.span),
      kind: ExpressionKind::Transform {
        callee: Box::new(operator_to_identifier(operator)),
        param: Box::new(transform_expression(*left)?),
      },
    }),
    param: Box::new(transform_expression(*right)?),
  })
}

fn transform_binary_pipe(
  left: Box<ConcreteExpression>,
  right: Box<ConcreteExpression>,
) -> CompilerResult<ExpressionKind<Spanned>> {
  Ok(ExpressionKind::Transform {
    callee: Box::new(transform_expression(*right)?),
    param: Box::new(transform_expression(*left)?),
  })
}

fn transform_binary_apply(
  left: Box<ConcreteExpression>,
  right: Box<ConcreteExpression>,
) -> CompilerResult<ExpressionKind<Spanned>> {
  Ok(ExpressionKind::Transform {
    callee: Box::new(transform_expression(*left)?),
    param: Box::new(transform_expression(*right)?),
  })
}
