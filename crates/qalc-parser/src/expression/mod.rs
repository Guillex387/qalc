mod atoms;
mod blocks;
mod numbers;
mod utils;

use pest::{iterators::Pairs, pratt_parser::PrattParser};
use qalc_ast::concrete_ast::{BinaryOperator, BinaryOperatorKind, ConcreteExpression, ConcreteExpressionKind};
use qalc_utils::CompilerResult;

use crate::expression::atoms::parse_atom_expression;
use crate::tokenizer::{Rule, Token};

lazy_static::lazy_static! {
  static ref PRATT_PARSER: PrattParser<Rule> = {
    use pest::pratt_parser::{ Assoc::*, Op };
    use Rule::*;

    // Precedence is defined lowest to highest
    PrattParser::new()
      .op(Op::infix(apply_operator, Right))
      .op(Op::infix(pipe_operator, Left))
      .op(Op::infix(logic_or_operator, Left))
      .op(Op::infix(logic_and_operator, Left))
      .op(
        Op::infix(or_operator, Left) |
          Op::infix(and_operator, Left) |
          Op::infix(xor_operator, Left)
      )
      .op(
        Op::infix(equal_operator, Left) |
          Op::infix(not_equal_operator, Left) |
          Op::infix(less_operator, Left) |
          Op::infix(greater_operator, Left) |
          Op::infix(less_equal_operator, Left) |
          Op::infix(greater_equal_operator, Left)
      )
      .op(Op::infix(add_operator, Left) | Op::infix(subtract_operator, Left))
      .op(
        Op::infix(multiply_operator, Left) |
          Op::infix(divide_operator, Left) |
          Op::infix(module_operator, Left)
      )
      .op(Op::infix(raised_operator, Right))
  };
}

fn parse_binary_operator(operator: Token) -> BinaryOperator {
  let kind = match operator.rule() {
    Rule::apply_operator => BinaryOperatorKind::Apply,
    Rule::pipe_operator => BinaryOperatorKind::Pipe,
    Rule::logic_or_operator => BinaryOperatorKind::LogicOr,
    Rule::logic_and_operator => BinaryOperatorKind::LogicAnd,
    Rule::or_operator => BinaryOperatorKind::Or,
    Rule::and_operator => BinaryOperatorKind::And,
    Rule::xor_operator => BinaryOperatorKind::Xor,
    Rule::equal_operator => BinaryOperatorKind::Equal,
    Rule::not_equal_operator => BinaryOperatorKind::NotEqual,
    Rule::less_equal_operator => BinaryOperatorKind::LessEqual,
    Rule::greater_equal_operator => BinaryOperatorKind::GreaterEqual,
    Rule::less_operator => BinaryOperatorKind::Less,
    Rule::greater_operator => BinaryOperatorKind::Greater,
    Rule::add_operator => BinaryOperatorKind::Add,
    Rule::subtract_operator => BinaryOperatorKind::Subtract,
    Rule::multiply_operator => BinaryOperatorKind::Multiply,
    Rule::divide_operator => BinaryOperatorKind::Divide,
    Rule::module_operator => BinaryOperatorKind::Module,
    Rule::raised_operator => BinaryOperatorKind::Raised,
    rule => unreachable!("Expected binary operator, found {:?}", rule),
  };

  BinaryOperator {
    kind,
    span: operator.span(),
  }
}

fn parse_binary_expression(
  left: ConcreteExpression,
  operator: Token,
  right: ConcreteExpression,
) -> CompilerResult<ConcreteExpression> {
  Ok(ConcreteExpression {
    span: left.span.join(&right.span),
    kind: ConcreteExpressionKind::Binary {
      left: Box::new(left),
      operator: parse_binary_operator(operator),
      right: Box::new(right),
    },
  })
}

pub fn parse_expression(token: Token) -> CompilerResult<ConcreteExpression> {
  let source = token.source().clone();
  let raw_token_iter: Pairs<Rule> = token.inner().into();

  PRATT_PARSER
    .map_primary(|raw_token| parse_atom_expression(Token::new(raw_token, &source)))
    .map_infix(|left, operator, right| {
      parse_binary_expression(left?, Token::new(operator, &source), right?)
    })
    .parse(raw_token_iter)
}
