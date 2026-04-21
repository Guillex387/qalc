mod binary;
mod linear;
mod perform;

use qalc_ast::concrete_ast::{Expression as CExpr, Pattern};
use qalc_ast::normalized_ast::{Expression as NExpr, ExpressionKind as NExprKind};
use qalc_utils::Span;

use crate::normalizer::expr::transform_expression;

pub use binary::*;
pub use linear::*;
pub use perform::*;

// Example (source):
//   match x { 1 -> 10, 2 -> 20 }
// Example (normalized):
//   if x == 1 then 10 else if x == 2 then 20 else ()
pub fn transform_match(
  value: Box<CExpr>,
  cases: Vec<(CExpr, CExpr)>,
  default: Option<Box<CExpr>>,
  span: Span,
) -> NExprKind {
  let value = Box::new(transform_expression(*value));
  let mut if_block = default
    .map(|expr| Box::new(transform_expression(*expr)))
    .unwrap_or(Box::new(NExpr {
      kind: NExprKind::Tuple(Vec::new()),
      span,
    }));

  for (check_value, return_value) in cases {
    let check_value = transform_expression(check_value);
    let return_value = transform_expression(return_value);

    let case_span = check_value.span.join(&return_value.span);
    let eq_call = NExpr {
      span: case_span.clone(),
      kind: NExprKind::Transform {
        callee: Box::new(NExpr {
          kind: NExprKind::Transform {
            callee: Box::new(NExpr {
              kind: NExprKind::Identifier("==".to_string()),
              span: case_span.clone(),
            }),
            param: value.clone(),
          },
          span: case_span.clone(),
        }),
        param: Box::new(check_value),
      },
    };

    if_block = Box::new(NExpr {
      span: case_span,
      kind: NExprKind::IfElseBlock {
        test: Box::new(eq_call),
        consequent: Box::new(return_value),
        alternative: if_block,
      },
    })
  }

  if_block.kind
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
pub fn transform_params_pattern_to_let(params: Vec<Pattern>, body: CExpr) -> (Vec<String>, NExpr) {
  let norm_params: Vec<String> = params
    .iter()
    .enumerate()
    .map(|(index, _)| format!("p{}", index).to_string())
    .collect();
  let span = params
    .first()
    .unwrap()
    .span
    .join(&params.last().unwrap().span);

  let new_body = NExpr {
    kind: NExprKind::LetBlock {
      assigments: params
        .into_iter()
        .zip(norm_params.clone())
        .map(|(pattern, name)| {
          (
            pattern.clone(),
            NExpr {
              kind: NExprKind::Identifier(name),
              span: pattern.span,
            },
          )
        })
        .collect(),
      value: Box::new(transform_expression(body)),
    },
    span,
  };

  (norm_params, new_body)
}

// Example (source):
//   someFunc a b c
// Example (normalized):
//   (((someFunc a) b) c)
pub fn transform_call(callee: Box<CExpr>, params: Vec<CExpr>) -> NExprKind {
  params
    .into_iter()
    .fold(transform_expression(*callee), |result, param| NExpr {
      span: result.span.join(&param.span),
      kind: NExprKind::Transform {
        callee: Box::new(result),
        param: Box::new(transform_expression(param)),
      },
    })
    .kind
}
