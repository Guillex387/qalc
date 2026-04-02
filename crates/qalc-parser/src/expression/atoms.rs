use pest::iterators::{ Pair, Pairs };
use qalc_ast::concrete_ast::{ Expression, ExpressionKind };

use crate::expression::{
  numbers::parse_number_literal,
  parse_binary_operator,
  parse_expression,
  parse_expression_pairs,
};
use crate::{ Rule, shared::token_to_span };

fn consume_expression(filename: &str, tokens: &mut Pairs<Rule>) -> Box<Expression> {
  Box::new(parse_expression(filename, tokens.next().unwrap()))
}

fn consume_back_expression(filename: &str, tokens: &mut Pairs<Rule>) -> Box<Expression> {
  Box::new(parse_expression(filename, tokens.next_back().unwrap()))
}

fn parse_let_block(filename: &str, mut tokens: Pairs<Rule>) -> ExpressionKind {
  let value = consume_back_expression(filename, &mut tokens);
  let assigments: Vec<Expression> = tokens
    .map(|token| parse_expression(filename, token))
    .collect();

  ExpressionKind::LetBlock { assigments, value }
}

fn parse_if_else_block(filename: &str, mut tokens: Pairs<Rule>) -> ExpressionKind {
  let test = consume_expression(filename, &mut tokens);
  let consequent = consume_expression(filename, &mut tokens);
  let alternative = consume_expression(filename, &mut tokens);

  ExpressionKind::IfElseBlock { test, consequent, alternative }
}

fn parse_match_block(filename: &str, mut tokens: Pairs<Rule>) -> ExpressionKind {
  let value = consume_expression(filename, &mut tokens);
  let has_default = tokens.len() % 2 != 0;
  let default: Option<Box<Expression>> = if has_default {
    Some(consume_back_expression(filename, &mut tokens))
  } else {
    None
  };
  let cases: Vec<(Expression, Expression)> = tokens
    .collect::<Vec<Pair<Rule>>>()
    .chunks(2)
    .filter_map(|tokens| {
      if tokens.len() == 2 {
        Some((
          parse_expression(filename, tokens[0].clone()),
          parse_expression(filename, tokens[1].clone()),
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

fn parse_perform_block(filename: &str, tokens: Pairs<Rule>) -> ExpressionKind {
  ExpressionKind::PerformBlock {
    sequence: tokens.map(|token| parse_expression(filename, token)).collect(),
  }
}

fn parse_linear_block(filename: &str, tokens: Pairs<Rule>) -> ExpressionKind {
  ExpressionKind::LinearBlock {
    expressions: tokens.map(|token| parse_expression(filename, token)).collect(),
  }
}

fn parse_call_expression(filename: &str, mut tokens: Pairs<Rule>) -> ExpressionKind {
  let callee = consume_expression(filename, &mut tokens);
  let params: Vec<Expression> = tokens
    .map(|token| parse_expression(filename, token))
    .collect();

  ExpressionKind::Call { callee, params }
}

fn parse_lambda_expression(filename: &str, mut tokens: Pairs<Rule>) -> ExpressionKind {
  let body = consume_back_expression(filename, &mut tokens);
  let params: Vec<String> = tokens
    .map(|identifier| identifier.as_str().to_string())
    .collect();

  ExpressionKind::Lambda { params, body }
}

fn parse_partial_op_left(filename: &str, mut tokens: Pairs<Rule>) -> ExpressionKind {
  let operator = parse_binary_operator(tokens.next().unwrap());
  let right = consume_expression(filename, &mut tokens);

  ExpressionKind::PartialLeftBinary { operator, right }
}

fn parse_partial_op_right(filename: &str, mut tokens: Pairs<Rule>) -> ExpressionKind {
  let left = consume_expression(filename, &mut tokens);
  let operator = parse_binary_operator(tokens.next().unwrap());

  ExpressionKind::PartialRightBinary { left, operator }
}

fn parse_assigment(filename: &str, mut tokens: Pairs<Rule>) -> ExpressionKind {
  let identifier = tokens.next().unwrap().as_str().to_string();
  let value = consume_expression(filename, &mut tokens);

  ExpressionKind::Assigment { identifier, value }
}

pub fn parse_atom_expression(filename: &str, primary: Pair<Rule>) -> Expression {
  let span = token_to_span(filename, &primary);
  let mut tokens = primary.clone().into_inner();
  let kind = match primary.as_rule() {
    Rule::let_block => parse_let_block(filename, tokens),
    Rule::if_else_block => parse_if_else_block(filename, tokens),
    Rule::match_block => parse_match_block(filename, tokens),
    Rule::perform_block => parse_perform_block(filename, tokens),
    Rule::linear_block => parse_linear_block(filename, tokens),
    Rule::call_expression => parse_call_expression(filename, tokens),
    Rule::lambda_expression => parse_lambda_expression(filename, tokens),
    Rule::partial_operation_left => parse_partial_op_left(filename, tokens),
    Rule::partial_operation_right => parse_partial_op_right(filename, tokens),
    Rule::assigment_expression => parse_assigment(filename, tokens),
    Rule::expression => parse_expression_pairs(filename, tokens).kind,
    | Rule::unknow_integer
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
    | Rule::f64_float => parse_number_literal(primary),
    Rule::simple_identifier => ExpressionKind::Identifier(primary.as_str().to_string()),
    Rule::operator_identifier =>
      ExpressionKind::Identifier(tokens.next().unwrap().as_str().to_string()),
    rule => unreachable!("Expected atom expression found {:?}", rule),
  };

  Expression { kind, span }
}
