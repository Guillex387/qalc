mod binary;
mod linear;
mod perform;

use qalc_ast::Spanned;
use qalc_ast::base_ast::{Expression, ExpressionKind, Pattern};
use qalc_ast::concrete_ast::ConcreteExpression;
use qalc_utils::{CompilerResult, Span};

use crate::normalizer::expr::transform_expression;

pub use binary::*;
pub use linear::*;
pub use perform::*;

// Example (source):
//   match x { 1 -> 10, 2 -> 20 }
// Example (normalized):
//   if x == 1 then 10 else if x == 2 then 20 else ()
pub fn transform_match(
  value: Box<ConcreteExpression>,
  cases: Vec<(ConcreteExpression, ConcreteExpression)>,
  default: Option<Box<ConcreteExpression>>,
  span: Span,
) -> CompilerResult<ExpressionKind<Spanned>> {
  let value = Box::new(transform_expression(*value)?);
  let default_case = match default {
    Some(concrete_expr) => Box::new(transform_expression(*concrete_expr)?),
    None => Box::new(Expression {
      kind: ExpressionKind::Tuple(Vec::new()),
      meta: span,
    }),
  };

  Ok(
    cases
      .into_iter()
      .try_fold(default_case, |if_block, (check_value, return_value)| {
        let check_value = transform_expression(check_value)?;
        let return_value = transform_expression(return_value)?;

        let case_span = check_value.meta.join(&return_value.meta);
        let eq_call = Expression {
          meta: case_span.clone(),
          kind: ExpressionKind::Transform {
            callee: Box::new(Expression {
              kind: ExpressionKind::Transform {
                callee: Box::new(Expression {
                  kind: ExpressionKind::Identifier("==".to_string()),
                  meta: case_span.clone(),
                }),
                param: value.clone(),
              },
              meta: case_span.clone(),
            }),
            param: Box::new(check_value),
          },
        };

        Ok(Box::new(Expression {
          meta: case_span,
          kind: ExpressionKind::IfElseBlock {
            test: Box::new(eq_call),
            consequent: Box::new(return_value),
            alternative: if_block,
          },
        }))
      })?
      .kind,
  )
}

// Example (source):
//   ((x, y)) -> x
// Example (normalized):
//   (p0) -> let (x, y) = p0 in x
// Example (source, function declaration):
//   add :: (i32, i32) -> i32
//   add (x, y) = x
// Example (normalized):
//   add :: (i32, i32) -> i32
//   add p0 = let (x, y) = p0 in x
pub fn transform_params_pattern_to_let(
  params: Vec<Pattern<Spanned>>,
  body: ConcreteExpression,
) -> CompilerResult<(Vec<String>, Expression<Spanned>)> {
  let norm_params: Vec<String> = params
    .iter()
    .enumerate()
    .map(|(index, _)| format!("p{}", index).to_string())
    .collect();
  let span = params
    .first()
    .unwrap()
    .meta
    .join(&params.last().unwrap().meta);

  let new_body = Expression {
    kind: ExpressionKind::LetBlock {
      strict: false,
      assigments: params
        .into_iter()
        .zip(norm_params.clone())
        .map(|(pattern, name)| {
          (
            pattern.clone(),
            Expression {
              kind: ExpressionKind::Identifier(name),
              meta: pattern.meta,
            },
          )
        })
        .collect(),
      value: Box::new(transform_expression(body)?),
    },
    meta: span,
  };

  Ok((norm_params, new_body))
}

// Example (source):
//   someFunc a b c
// Example (normalized):
//   (((someFunc a) b) c)
pub fn transform_call(
  callee: Box<ConcreteExpression>,
  params: Vec<ConcreteExpression>,
) -> CompilerResult<ExpressionKind<Spanned>> {
  Ok(
    params
      .into_iter()
      .try_fold(transform_expression(*callee)?, |result, param| {
        Ok(Expression {
          meta: result.meta.join(&param.span),
          kind: ExpressionKind::Transform {
            callee: Box::new(result),
            param: Box::new(transform_expression(param)?),
          },
        })
      })?
      .kind,
  )
}
