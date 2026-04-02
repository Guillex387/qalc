mod atoms;
mod numbers;

use pest::{ iterators::{ Pair, Pairs }, pratt_parser::PrattParser };
use qalc_ast::concrete_ast::{ Expression, ExpressionKind, BinaryOperator };
use qalc_utils::Span;

use crate::{ Rule, expression::atoms::parse_atom_expression };

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

fn parse_binary_operator(operator: Pair<Rule>) -> BinaryOperator {
  match operator.as_rule() {
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

fn parse_binary_expression(
  filename: &str,
  left: Expression,
  operator: Pair<Rule>,
  right: Expression
) -> Expression {
  Expression {
    span: Span::new(filename, left.span.start, right.span.end),
    kind: ExpressionKind::Binary {
      left: Box::new(left),
      operator: parse_binary_operator(operator),
      right: Box::new(right),
    },
  }
}

pub fn parse_expression_pairs(filename: &str, pairs: Pairs<Rule>) -> Expression {
  PRATT_PARSER.map_primary(|token| parse_atom_expression(filename, token))
    .map_infix(|left, operator, right|
      parse_binary_expression(filename, left, operator, right)
    )
    .parse(pairs)
}

pub fn parse_expression(filename: &str, token: Pair<Rule>) -> Expression {
  parse_expression_pairs(filename, Pairs::single(token))
}
