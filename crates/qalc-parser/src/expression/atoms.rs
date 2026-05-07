use qalc_ast::Spanned;
use qalc_ast::base_ast::Pattern;
use qalc_ast::concrete_ast::{ConcreteExpression, ConcreteExpressionKind, ConcreteSliceExpression};
use qalc_utils::CompilerResult;

use crate::expression::blocks::{
  parse_if_else_block, parse_let_block, parse_linear_block, parse_match_block, parse_perform_block,
};
use crate::expression::utils::{consume_atom, consume_back_expression, consume_expression};
use crate::expression::{numbers::parse_number_literal, parse_binary_operator, parse_expression};
use crate::pattern::parse_pattern;
use crate::tokenizer::{Rule, Token, TokensIter};

fn parse_call_expression(mut tokens: TokensIter) -> CompilerResult<ConcreteExpressionKind> {
  let callee = consume_atom(&mut tokens)?;
  let params = tokens
    .map(parse_atom_expression)
    .collect::<CompilerResult<Vec<ConcreteExpression>>>()?;

  Ok(ConcreteExpressionKind::Call { callee, params })
}

fn parse_lambda_expression(mut tokens: TokensIter) -> CompilerResult<ConcreteExpressionKind> {
  let body = consume_back_expression(&mut tokens)?;
  let params: Vec<Pattern<Spanned>> = tokens.map(parse_pattern).collect();

  Ok(ConcreteExpressionKind::Lambda { params, body })
}

fn parse_partial_op_left(mut tokens: TokensIter) -> CompilerResult<ConcreteExpressionKind> {
  let operator = parse_binary_operator(tokens.eat());
  let right = consume_atom(&mut tokens)?;

  Ok(ConcreteExpressionKind::PartialLeftBinary { operator, right })
}

fn parse_partial_op_right(mut tokens: TokensIter) -> CompilerResult<ConcreteExpressionKind> {
  let left = consume_atom(&mut tokens)?;
  let operator = parse_binary_operator(tokens.eat());

  Ok(ConcreteExpressionKind::PartialRightBinary { left, operator })
}

fn parse_tuple_expression(tokens: TokensIter) -> CompilerResult<ConcreteExpressionKind> {
  Ok(ConcreteExpressionKind::Tuple(
    tokens
      .map(parse_expression)
      .collect::<CompilerResult<Vec<ConcreteExpression>>>()?,
  ))
}

fn parse_slice_items_expression(tokens: TokensIter) -> CompilerResult<ConcreteExpressionKind> {
  Ok(ConcreteExpressionKind::Slice(
    ConcreteSliceExpression::Items(
      tokens
        .map(parse_expression)
        .collect::<CompilerResult<Vec<ConcreteExpression>>>()?,
    ),
  ))
}

fn parse_slice_range_expression(mut tokens: TokensIter) -> CompilerResult<ConcreteExpressionKind> {
  Ok(ConcreteExpressionKind::Slice(
    ConcreteSliceExpression::Range(
      consume_expression(&mut tokens)?,
      consume_expression(&mut tokens)?,
    ),
  ))
}

pub fn parse_atom_expression(token: Token) -> CompilerResult<ConcreteExpression> {
  let span = token.span();
  let rule = token.rule();
  let mut tokens = token.clone().inner();
  let kind = match rule {
    Rule::let_block => parse_let_block(tokens)?,
    Rule::if_else_block => parse_if_else_block(tokens)?,
    Rule::match_block => parse_match_block(tokens)?,
    Rule::perform_block => parse_perform_block(tokens)?,
    Rule::linear_block => parse_linear_block(tokens)?,
    Rule::call_expression => parse_call_expression(tokens)?,
    Rule::lambda_expression => parse_lambda_expression(tokens)?,
    Rule::partial_operation_left => parse_partial_op_left(tokens)?,
    Rule::partial_operation_right => parse_partial_op_right(tokens)?,
    Rule::tuple_expression => parse_tuple_expression(tokens)?,
    Rule::slice_items_expression => parse_slice_items_expression(tokens)?,
    Rule::slice_range_expression => parse_slice_range_expression(tokens)?,
    Rule::expression => parse_expression(token)?.kind,
    Rule::unknow_integer
    | Rule::i8_integer
    | Rule::i16_integer
    | Rule::i32_integer
    | Rule::i64_integer
    | Rule::u8_integer
    | Rule::u16_integer
    | Rule::u32_integer
    | Rule::u64_integer
    | Rule::unknow_float
    | Rule::f32_float
    | Rule::f64_float => parse_number_literal(token),
    Rule::simple_identifier => ConcreteExpressionKind::Identifier(token.to_string()),
    Rule::operator_identifier => ConcreteExpressionKind::Identifier(tokens.eat().to_string()),
    rule => unreachable!("Expected atom expression found {:?}", rule),
  };

  Ok(ConcreteExpression { kind, span })
}
