use qalc_ast::concrete_ast::{self, Expression as CExpr, ExpressionKind as CExprKind};
use qalc_ast::normalized_ast::{self, ExpressionKind as NExprKind};

use crate::normalizer::expr::transform_expression;

pub fn adapt_let_expression(assigments: Vec<CExpr>, value: Box<CExpr>) -> NExprKind {
  let assigments = assigments
    .into_iter()
    .map(|assigment| {
      if let CExprKind::Assigment {
        identifier, value, ..
      } = assigment.kind
      {
        let value = transform_expression(*value);
        (identifier, value)
      } else {
        unreachable!();
      }
    })
    .collect();
  let value = Box::new(transform_expression(*value));

  NExprKind::LetBlock {
    strict: false,
    assigments,
    value,
  }
}

pub fn adapt_ifelse_block(
  test: Box<CExpr>,
  consequent: Box<CExpr>,
  alternative: Box<CExpr>,
) -> NExprKind {
  let test = Box::new(transform_expression(*test));
  let consequent = Box::new(transform_expression(*consequent));
  let alternative = Box::new(transform_expression(*alternative));

  NExprKind::IfElseBlock {
    test,
    consequent,
    alternative,
  }
}

pub fn adapt_slice(slice: concrete_ast::SliceExpression) -> NExprKind {
  let slice_expr = match slice {
    concrete_ast::SliceExpression::Items(items) => {
      normalized_ast::SliceExpression::Items(items.into_iter().map(transform_expression).collect())
    }
    concrete_ast::SliceExpression::Range(start, end) => {
      let start = Box::new(transform_expression(*start));
      let end = Box::new(transform_expression(*end));

      normalized_ast::SliceExpression::Range(start, end)
    }
  };

  NExprKind::Slice(slice_expr)
}
