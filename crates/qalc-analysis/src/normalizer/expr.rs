use qalc_ast::concrete_ast::{Expression as CExpr, ExpressionKind as CExprKind};
use qalc_ast::normalized_ast::{Expression as NExpr, ExpressionKind as NExprKind};

use crate::normalizer::adapters::{adapt_ifelse_block, adapt_let_expression, adapt_slice};
use crate::normalizer::desugar::{
  transform_binary, transform_linear_block, transform_match, transform_params_pattern_to_let,
  transform_partial_left, transform_partial_right, transform_perform_block,
};

pub fn transform_expression(expression: CExpr) -> NExpr {
  let span = expression.span;
  let kind = match expression.kind {
    CExprKind::LetBlock { assigments, value } => adapt_let_expression(assigments, value),
    CExprKind::IfElseBlock {
      test,
      consequent,
      alternative,
    } => adapt_ifelse_block(test, consequent, alternative),
    CExprKind::MatchBlock {
      value,
      cases,
      default,
    } => transform_match(value, cases, default, span.clone()),
    CExprKind::PerformBlock { sequence, value } => transform_perform_block(sequence, value),
    CExprKind::LinearBlock { sequence, value } => transform_linear_block(sequence, value),
    CExprKind::Binary {
      left,
      operator,
      right,
    } => transform_binary(left, operator, right),
    CExprKind::PartialLeftBinary { operator, right } => {
      transform_partial_left(operator, right, span.clone())
    }
    CExprKind::PartialRightBinary { left, operator } => transform_partial_right(left, operator),
    CExprKind::Call { callee, params } => {
      let callee = Box::new(transform_expression(*callee));
      let params = params.into_iter().map(transform_expression).collect();

      NExprKind::Call { callee, params }
    }
    CExprKind::Lambda { params, body } => {
      let (params, body) = transform_params_pattern_to_let(params, *body);

      NExprKind::Lambda {
        params,
        body: Box::new(body),
      }
    }
    CExprKind::Tuple(tuple) => {
      NExprKind::Tuple(tuple.into_iter().map(transform_expression).collect())
    }
    CExprKind::Slice(slice) => adapt_slice(slice),
    CExprKind::Identifier(identifier) => NExprKind::Identifier(identifier),
    CExprKind::IntegerLiteral { number, suffix } => NExprKind::IntegerLiteral { number, suffix },
    CExprKind::FloatLiteral { number, suffix } => NExprKind::FloatLiteral { number, suffix },
    CExprKind::Assigment { .. } => unreachable!(),
  };

  NExpr { kind, span }
}
