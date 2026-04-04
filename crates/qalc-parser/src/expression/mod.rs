mod atoms;
mod numbers;

use pest::{iterators::Pairs, pratt_parser::PrattParser};
use qalc_ast::concrete_ast::{BinaryOperator, Expression, ExpressionKind};

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
  match operator.rule() {
    Rule::apply_operator => BinaryOperator::Apply,
    Rule::pipe_operator => BinaryOperator::Pipe,
    Rule::logic_or_operator => BinaryOperator::LogicOr,
    Rule::logic_and_operator => BinaryOperator::LogicAnd,
    Rule::or_operator => BinaryOperator::Or,
    Rule::and_operator => BinaryOperator::And,
    Rule::xor_operator => BinaryOperator::Xor,
    Rule::equal_operator => BinaryOperator::Equal,
    Rule::not_equal_operator => BinaryOperator::NotEqual,
    Rule::less_equal_operator => BinaryOperator::LessEqual,
    Rule::greater_equal_operator => BinaryOperator::GreaterEqual,
    Rule::less_operator => BinaryOperator::Less,
    Rule::greater_operator => BinaryOperator::Greater,
    Rule::add_operator => BinaryOperator::Add,
    Rule::subtract_operator => BinaryOperator::Subtract,
    Rule::multiply_operator => BinaryOperator::Multiply,
    Rule::divide_operator => BinaryOperator::Divide,
    Rule::module_operator => BinaryOperator::Module,
    Rule::raised_operator => BinaryOperator::Raised,
    rule => unreachable!("Expected binary operator, found {:?}", rule),
  }
}

fn parse_binary_expression(left: Expression, operator: Token, right: Expression) -> Expression {
  Expression {
    span: left.span.join(&right.span),
    kind: ExpressionKind::Binary {
      left: Box::new(left),
      operator: parse_binary_operator(operator),
      right: Box::new(right),
    },
  }
}

pub fn parse_expression(token: Token) -> Expression {
  let source_file = token.source_file();
  let raw_token_iter: Pairs<Rule> = token.inner().into();

  PRATT_PARSER
    .map_primary(|raw_token| parse_atom_expression(Token::new(raw_token, source_file)))
    .map_infix(|left, operator, right| {
      parse_binary_expression(left, Token::new(operator, source_file), right)
    })
    .parse(raw_token_iter)
}
