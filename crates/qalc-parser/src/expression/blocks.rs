use qalc_ast::concrete_ast::{Expression, ExpressionKind};
use qalc_utils::{CompilerResult, Diagnostic};

use crate::expression::parse_expression;
use crate::expression::utils::{consume_back_expression, consume_expression};
use crate::pattern::parse_pattern;
use crate::tokenizer::{Rule, Token, TokensIter};

fn parse_assigment(token: Token) -> CompilerResult<Expression> {
  let span = token.span();
  let rule = token.rule();
  let mut tokens = token.inner();

  let identifier = parse_pattern(tokens.eat());
  let value = consume_expression(&mut tokens)?;
  let monadic = rule == Rule::monadic_assigment;
  let kind = ExpressionKind::Assigment {
    identifier,
    value,
    monadic,
  };

  Ok(Expression { kind, span })
}

pub fn parse_let_block(mut tokens: TokensIter) -> CompilerResult<ExpressionKind> {
  let value = consume_back_expression(&mut tokens)?;
  let assigments = tokens
    .map(parse_assigment)
    .collect::<CompilerResult<Vec<Expression>>>()?;

  Ok(ExpressionKind::LetBlock { assigments, value })
}

pub fn parse_if_else_block(mut tokens: TokensIter) -> CompilerResult<ExpressionKind> {
  let test = consume_expression(&mut tokens)?;
  let consequent = consume_expression(&mut tokens)?;
  let alternative = consume_expression(&mut tokens)?;

  Ok(ExpressionKind::IfElseBlock {
    test,
    consequent,
    alternative,
  })
}

pub fn parse_match_block(mut tokens: TokensIter) -> CompilerResult<ExpressionKind> {
  let value = consume_expression(&mut tokens)?;

  let cases = tokens
    .collect::<Vec<Token>>()
    .chunks(2)
    .map(|tokens| {
      if let [case, value] = tokens {
        Ok((
          parse_expression(case.clone())?,
          parse_expression(value.clone())?,
        ))
      } else {
        unreachable!("Expected case-value format, found {:?}", tokens)
      }
    })
    .collect::<CompilerResult<Vec<(Expression, Expression)>>>()?;

  let (default_cases, cases): (Vec<(Expression, Expression)>, Vec<(Expression, Expression)>) =
    cases.iter().cloned().partition(|(predicate, _)| {
      if let ExpressionKind::Identifier(identifier) = &predicate.kind {
        identifier == "_"
      } else {
        false
      }
    });

  if default_cases.len() > 1 {
    let first_default_span = &default_cases.first().unwrap().0.span;
    let last_default_span = &default_cases.last().unwrap().1.span;

    return Err(Diagnostic::error(
      "multiple default cases in match",
      "multiple default cases found here",
      first_default_span.join(&last_default_span),
    ));
  }

  let default = default_cases
    .first()
    .map(|(_, value)| Box::new(value.clone()));

  Ok(ExpressionKind::MatchBlock {
    value,
    cases,
    default,
  })
}

fn parse_monadic_block(
  tokens: TokensIter,
) -> CompilerResult<(Vec<Expression>, Option<Box<Expression>>)> {
  let (sequence_tokens, return_token): (Vec<Token>, Vec<Token>) =
    tokens.partition(|token| token.rule() != Rule::return_expression);
  let return_token = return_token.first().map(ToOwned::to_owned);

  let sequence = sequence_tokens
    .into_iter()
    .map(|token| match token.rule() {
      Rule::monadic_assigment | Rule::assigment_expression => parse_assigment(token),
      Rule::expression => parse_expression(token),
      _ => unreachable!(
        "Expected expression or assigment expression, found {:?}",
        token.rule()
      ),
    })
    .collect::<CompilerResult<Vec<Expression>>>()?;
  let value = return_token
    .map(|token| parse_expression(token.inner().eat()))
    .transpose()?
    .map(Box::new);

  Ok((sequence, value))
}

pub fn parse_perform_block(tokens: TokensIter) -> CompilerResult<ExpressionKind> {
  let (sequence, value) = parse_monadic_block(tokens)?;

  Ok(ExpressionKind::PerformBlock { sequence, value })
}

pub fn parse_linear_block(tokens: TokensIter) -> CompilerResult<ExpressionKind> {
  let (sequence, value) = parse_monadic_block(tokens)?;

  Ok(ExpressionKind::LinearBlock { sequence, value })
}
