use qalc_ast::concrete_ast::{Expression, ExpressionKind};

use crate::expression::parse_expression;
use crate::expression::utils::{consume_back_expression, consume_expression};
use crate::pattern::parse_pattern;
use crate::tokenizer::{Rule, Token, TokensIter};

fn parse_assigment(token: Token) -> Expression {
  let span = token.span();
  let mut tokens = token.inner();
  let identifier = parse_pattern(tokens.eat());
  let value = consume_expression(&mut tokens);
  let kind = ExpressionKind::Assigment { identifier, value };

  Expression { kind, span }
}

pub fn parse_let_block(mut tokens: TokensIter) -> ExpressionKind {
  let value = consume_back_expression(&mut tokens);
  let assigments: Vec<Expression> = tokens.map(parse_assigment).collect();

  ExpressionKind::LetBlock { assigments, value }
}

pub fn parse_if_else_block(mut tokens: TokensIter) -> ExpressionKind {
  let test = consume_expression(&mut tokens);
  let consequent = consume_expression(&mut tokens);
  let alternative = consume_expression(&mut tokens);

  ExpressionKind::IfElseBlock {
    test,
    consequent,
    alternative,
  }
}

pub fn parse_match_block(mut tokens: TokensIter) -> ExpressionKind {
  let value = consume_expression(&mut tokens);

  let cases: Vec<(Expression, Expression)> = tokens
    .collect::<Vec<Token>>()
    .chunks(2)
    .filter_map(|tokens| {
      if let [case, value] = tokens {
        Some((
          parse_expression(case.clone()),
          parse_expression(value.clone()),
        ))
      } else {
        None
      }
    })
    .collect();

  let (default_cases, cases): (Vec<(Expression, Expression)>, Vec<(Expression, Expression)>) =
    cases.iter().cloned().partition(|(predicate, _)| {
      if let ExpressionKind::Identifier(identifier) = &predicate.kind {
        identifier == "_"
      } else {
        false
      }
    });

  if default_cases.len() > 1 {
    todo!("Puts an error with multiple labels here");
  }

  let default = default_cases
    .first()
    .map(|(_, value)| Box::new(value.clone()));

  ExpressionKind::MatchBlock {
    value,
    cases,
    default,
  }
}

pub fn parse_perform_block(tokens: TokensIter) -> ExpressionKind {
  ExpressionKind::PerformBlock {
    sequence: tokens
      .map(|token| match token.rule() {
        Rule::assigment_expression => parse_assigment(token),
        Rule::expression => parse_expression(token),
        _ => unreachable!(
          "Expected expression or assigment expression, found {:?}",
          token.rule()
        ),
      })
      .collect(),
  }
}

pub fn parse_linear_block(tokens: TokensIter) -> ExpressionKind {
  ExpressionKind::LinearBlock {
    expressions: tokens
      .map(|token| match token.rule() {
        Rule::assigment_expression => parse_assigment(token),
        Rule::expression => parse_expression(token),
        _ => unreachable!(
          "Expected expression or assigment expression, found {:?}",
          token.rule()
        ),
      })
      .collect(),
  }
}
