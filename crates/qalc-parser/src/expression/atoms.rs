use pest::iterators::{ Pair, Pairs };
use qalc_ast::concrete_ast::{ Expression, ExpressionKind };

use crate::expression::{
  numbers::parse_number_literal,
  parse_binary_operator,
  parse_expression,
  parse_expression_pairs,
};
use crate::{ Rule, shared::token_to_span };

fn consume_expression(tokens: &mut Pairs<Rule>) -> Box<Expression> {
  Box::new(parse_expression(tokens.next().unwrap()))
}

fn consume_back_expression(tokens: &mut Pairs<Rule>) -> Box<Expression> {
  Box::new(parse_expression(tokens.next_back().unwrap()))
}

fn parse_let_block(mut tokens: Pairs<Rule>) -> ExpressionKind {
  let value = consume_back_expression(&mut tokens);
  let assigments: Vec<Expression> = tokens.map(parse_expression).collect();

  ExpressionKind::LetBlock { assigments, value }
}

fn parse_if_else_block(mut tokens: Pairs<Rule>) -> ExpressionKind {
  let test = consume_expression(&mut tokens);
  let consequent = consume_expression(&mut tokens);
  let alternative = consume_expression(&mut tokens);

  ExpressionKind::IfElseBlock { test, consequent, alternative }
}

fn parse_match_block(mut tokens: Pairs<Rule>) -> ExpressionKind {
  let value = consume_expression(&mut tokens);
  let has_default = tokens.len() % 2 != 0;
  let default: Option<Box<Expression>> = if has_default { Some(consume_back_expression(&mut tokens)) } else { None };
  let cases: Vec<(Expression, Expression)> = tokens
    .collect::<Vec<Pair<Rule>>>()
    .chunks(2)
    .filter_map(|tokens| {
      if tokens.len() == 2 {
        Some((parse_expression(tokens[0].clone()), parse_expression(tokens[1].clone())))
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

fn parse_perform_block(tokens: Pairs<Rule>) -> ExpressionKind {
  ExpressionKind::PerformBlock { sequence: tokens.map(parse_expression).collect() }
}

fn parse_linear_block(tokens: Pairs<Rule>) -> ExpressionKind {
  ExpressionKind::LinearBlock { expressions: tokens.map(parse_expression).collect() }
}

fn parse_call_expression(mut tokens: Pairs<Rule>) -> ExpressionKind {
  let callee = consume_expression(&mut tokens);
  let params: Vec<Expression> = tokens.map(parse_expression).collect();

  ExpressionKind::Call { callee, params }
}

fn parse_lambda_expression(mut tokens: Pairs<Rule>) -> ExpressionKind {
  let body = consume_back_expression(&mut tokens);
  let params: Vec<String> = tokens.map(|identifier| identifier.as_str().to_string()).collect();

  ExpressionKind::Lambda { params, body }
}

fn parse_partial_op_left(mut tokens: Pairs<Rule>) -> ExpressionKind {
  let operator = parse_binary_operator(tokens.next().unwrap());
  let right = consume_expression(&mut tokens);

  ExpressionKind::PartialLeftBinary { operator, right }
}

fn parse_partial_op_right(mut tokens: Pairs<Rule>) -> ExpressionKind {
  let left = consume_expression(&mut tokens);
  let operator = parse_binary_operator(tokens.next().unwrap());

  ExpressionKind::PartialRightBinary { left, operator }
}

fn parse_assigment(mut tokens: Pairs<Rule>) -> ExpressionKind {
  let identifier = tokens.next().unwrap().as_str().to_string();
  let value = consume_expression(&mut tokens);

  ExpressionKind::Assigment { identifier, value }
}

pub fn parse_atom_expression(primary: Pair<Rule>) -> Expression {
  let span = token_to_span(&primary);
  let mut tokens = primary.clone().into_inner();
  let kind = match primary.as_rule() {
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
    Rule::expression => parse_expression_pairs(tokens).kind,
    | Rule::unknow_integer
    | Rule::i8_integer
    | Rule::i16_integer
    | Rule::i32_integer
    | Rule::i64_integer
    | Rule::u8_integer
    | Rule::u16_integer
    | Rule::u32_integer
    | Rule::u64_integer => parse_number_literal(primary),
    Rule::simple_identifier => ExpressionKind::Identifier(primary.as_str().to_string()),
    Rule::operator_identifier => ExpressionKind::Identifier(tokens.next().unwrap().as_str().to_string()),
    rule => unreachable!("Expected atom expression found {:?}", rule),
  };

  Expression { kind, span }
}
