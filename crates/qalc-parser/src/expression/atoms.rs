use qalc_ast::concrete_ast::{Expression, ExpressionKind};

use crate::Rule;
use crate::expression::{numbers::parse_number_literal, parse_binary_operator, parse_expression};
use crate::{Token, TokensIter};

fn consume_expression(tokens: &mut TokensIter) -> Box<Expression> {
  Box::new(parse_expression(tokens.eat()))
}

fn consume_back_expression(tokens: &mut TokensIter) -> Box<Expression> {
  Box::new(parse_expression(tokens.eat_back()))
}

fn parse_let_block(mut tokens: TokensIter) -> ExpressionKind {
  let value = consume_back_expression(&mut tokens);
  let assigments: Vec<Expression> = tokens.map(parse_expression).collect();

  ExpressionKind::LetBlock { assigments, value }
}

fn parse_if_else_block(mut tokens: TokensIter) -> ExpressionKind {
  let test = consume_expression(&mut tokens);
  let consequent = consume_expression(&mut tokens);
  let alternative = consume_expression(&mut tokens);

  ExpressionKind::IfElseBlock {
    test,
    consequent,
    alternative,
  }
}

fn parse_match_block(mut tokens: TokensIter) -> ExpressionKind {
  let value = consume_expression(&mut tokens);
  let has_default = tokens.len() % 2 != 0;
  let default: Option<Box<Expression>> = if has_default {
    Some(consume_back_expression(&mut tokens))
  } else {
    None
  };

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

  ExpressionKind::MatchBlock {
    value,
    cases,
    default,
  }
}

fn parse_perform_block(tokens: TokensIter) -> ExpressionKind {
  ExpressionKind::PerformBlock {
    sequence: tokens.map(parse_expression).collect(),
  }
}

fn parse_linear_block(tokens: TokensIter) -> ExpressionKind {
  ExpressionKind::LinearBlock {
    expressions: tokens.map(parse_expression).collect(),
  }
}

fn parse_call_expression(mut tokens: TokensIter) -> ExpressionKind {
  let callee = consume_expression(&mut tokens);
  let params: Vec<Expression> = tokens.map(parse_expression).collect();

  ExpressionKind::Call { callee, params }
}

fn parse_lambda_expression(mut tokens: TokensIter) -> ExpressionKind {
  let body = consume_back_expression(&mut tokens);
  let params: Vec<String> = tokens.map(|token| token.to_string()).collect();

  ExpressionKind::Lambda { params, body }
}

fn parse_partial_op_left(mut tokens: TokensIter) -> ExpressionKind {
  let operator = parse_binary_operator(tokens.eat());
  let right = consume_expression(&mut tokens);

  ExpressionKind::PartialLeftBinary { operator, right }
}

fn parse_partial_op_right(mut tokens: TokensIter) -> ExpressionKind {
  let left = consume_expression(&mut tokens);
  let operator = parse_binary_operator(tokens.eat());

  ExpressionKind::PartialRightBinary { left, operator }
}

fn parse_assigment(mut tokens: TokensIter) -> ExpressionKind {
  let identifier = tokens.eat().to_string();
  let value = consume_expression(&mut tokens);

  ExpressionKind::Assigment { identifier, value }
}

pub fn parse_atom_expression(token: Token) -> Expression {
  let span = token.span();
  let rule = token.rule();
  let mut tokens = token.clone().inner();
  let kind = match rule {
    Rule::let_block => parse_let_block(tokens),
    Rule::if_else_block => parse_if_else_block(tokens),
    Rule::match_block => parse_match_block(tokens),
    Rule::perform_block => parse_perform_block(tokens),
    Rule::linear_block => parse_linear_block(tokens),
    Rule::call_expression => parse_call_expression(tokens),
    Rule::lambda_expression => parse_lambda_expression(tokens),
    Rule::partial_operation_left => parse_partial_op_left(tokens),
    Rule::partial_operation_right => parse_partial_op_right(tokens),
    Rule::assigment_expression => parse_assigment(tokens),
    Rule::expression => parse_expression(token).kind,
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
    Rule::simple_identifier => ExpressionKind::Identifier(token.to_string()),
    Rule::operator_identifier => ExpressionKind::Identifier(tokens.eat().to_string()),
    rule => unreachable!("Expected atom expression found {:?}", rule),
  };

  Expression { kind, span }
}
