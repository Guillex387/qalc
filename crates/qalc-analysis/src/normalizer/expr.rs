use qalc_ast::Spanned;
use qalc_ast::base_ast::{Expression, ExpressionKind};
use qalc_ast::concrete_ast::{ConcreteExpression, ConcreteExpressionKind};
use qalc_utils::CompilerResult;

use crate::normalizer::adapters::{adapt_ifelse_block, adapt_let_expression, adapt_slice};
use crate::normalizer::desugar::{
  transform_binary, transform_call, transform_linear_block, transform_match,
  transform_params_pattern_to_let, transform_partial_left, transform_partial_right,
  transform_perform_block,
};

pub fn transform_expression(expression: ConcreteExpression) -> CompilerResult<Expression<Spanned>> {
  let span = expression.span;
  let kind = match expression.kind {
    ConcreteExpressionKind::LetBlock { assigments, value } => {
      adapt_let_expression(assigments, value)?
    }
    ConcreteExpressionKind::IfElseBlock {
      test,
      consequent,
      alternative,
    } => adapt_ifelse_block(test, consequent, alternative)?,
    ConcreteExpressionKind::MatchBlock {
      value,
      cases,
      default,
    } => transform_match(value, cases, default, span.clone())?,
    ConcreteExpressionKind::PerformBlock { sequence, value } => {
      transform_perform_block(sequence, value)?
    }
    ConcreteExpressionKind::LinearBlock { sequence, value } => {
      transform_linear_block(sequence, value)?
    }
    ConcreteExpressionKind::Binary {
      left,
      operator,
      right,
    } => transform_binary(left, operator, right)?,
    ConcreteExpressionKind::PartialLeftBinary { operator, right } => {
      transform_partial_left(operator, right, span.clone())?
    }
    ConcreteExpressionKind::PartialRightBinary { left, operator } => {
      transform_partial_right(left, operator)?
    }
    ConcreteExpressionKind::Call { callee, params } => transform_call(callee, params)?,
    ConcreteExpressionKind::Lambda { params, body } => {
      let (params, body) = transform_params_pattern_to_let(params, *body)?;

      ExpressionKind::Lambda {
        params,
        body: Box::new(body),
      }
    }
    ConcreteExpressionKind::Tuple(tuple) => ExpressionKind::Tuple(
      tuple
        .into_iter()
        .map(transform_expression)
        .collect::<CompilerResult<_>>()?,
    ),
    ConcreteExpressionKind::Slice(slice) => adapt_slice(slice)?,
    ConcreteExpressionKind::Identifier(identifier) => ExpressionKind::Identifier(identifier),
    ConcreteExpressionKind::IntegerLiteral { number, suffix } => {
      ExpressionKind::IntegerLiteral { number, suffix }
    }
    ConcreteExpressionKind::FloatLiteral { number, suffix } => {
      ExpressionKind::FloatLiteral { number, suffix }
    }
    ConcreteExpressionKind::Assigment { .. } => unreachable!(),
  };

  Ok(Expression { kind, meta: span })
}
