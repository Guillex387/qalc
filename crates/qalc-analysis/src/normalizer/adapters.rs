use qalc_ast::Spanned;
use qalc_ast::base_ast::{ExpressionKind, SliceExpression};
use qalc_ast::concrete_ast::{ConcreteExpression, ConcreteExpressionKind, ConcreteSliceExpression};
use qalc_utils::CompilerResult;

use crate::normalizer::expr::transform_expression;

pub fn adapt_let_expression(
  assigments: Vec<ConcreteExpression>,
  value: Box<ConcreteExpression>,
) -> CompilerResult<ExpressionKind<Spanned>> {
  let assigments = assigments
    .into_iter()
    .map(|assigment| {
      if let ConcreteExpressionKind::Assigment {
        identifier, value, ..
      } = assigment.kind
      {
        let value = transform_expression(*value)?;
        Ok((identifier, value))
      } else {
        unreachable!();
      }
    })
    .collect::<CompilerResult<_>>()?;
  let value = Box::new(transform_expression(*value)?);

  Ok(ExpressionKind::LetBlock {
    strict: false,
    assigments,
    value,
  })
}

pub fn adapt_ifelse_block(
  test: Box<ConcreteExpression>,
  consequent: Box<ConcreteExpression>,
  alternative: Box<ConcreteExpression>,
) -> CompilerResult<ExpressionKind<Spanned>> {
  let test = Box::new(transform_expression(*test)?);
  let consequent = Box::new(transform_expression(*consequent)?);
  let alternative = Box::new(transform_expression(*alternative)?);

  Ok(ExpressionKind::IfElseBlock {
    test,
    consequent,
    alternative,
  })
}

pub fn adapt_slice(slice: ConcreteSliceExpression) -> CompilerResult<ExpressionKind<Spanned>> {
  let slice_expr = match slice {
    ConcreteSliceExpression::Items(items) => SliceExpression::Items(
      items
        .into_iter()
        .map(transform_expression)
        .collect::<CompilerResult<_>>()?,
    ),
    ConcreteSliceExpression::Range(start, end) => {
      let start = Box::new(transform_expression(*start)?);
      let end = Box::new(transform_expression(*end)?);

      SliceExpression::Range(start, end)
    }
  };

  Ok(ExpressionKind::Slice(slice_expr))
}
