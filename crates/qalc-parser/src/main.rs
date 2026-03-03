use std::env::args;

use pest::{Parser, iterators::{Pair, Pairs}, pratt_parser::PrattParser};
use pest_derive::Parser;
use qalc_ast::concrete_ast::{BinaryOperator, Expression, ExpressionKind, NumberSuffix, Span};

#[allow(dead_code)]
#[derive(Parser)]
#[grammar = "grammar.pest"]
struct QalcGrammar;

lazy_static::lazy_static! {
  static ref PRATT_PARSER: PrattParser<Rule> = {
    use pest::pratt_parser::{Assoc::*, Op};
    use Rule::*;

    // Precedence is defined lowest to highest
    PrattParser::new()
      .op(Op::infix(apply_operator, Right))
      .op(Op::infix(pipe_operator, Left))
      .op(Op::infix(logic_or_operator, Left))
      .op(Op::infix(logic_and_operator, Left))
      .op(Op::infix(or_operator, Left) | Op::infix(and_operator, Left) | Op::infix(xor_operator, Left))
      .op(Op::infix(equal_operator, Left) | Op::infix(not_equal_operator, Left) | Op::infix(less_operator, Left) | Op::infix(greater_operator, Left) | Op::infix(less_equal_operator, Left) | Op::infix(greater_equal_operator, Left))
      .op(Op::infix(add_operator, Left) | Op::infix(subtract_operator, Left))
      .op(Op::infix(multiply_operator, Left) | Op::infix(divide_operator, Left) | Op::infix(module_operator, Left))
      .op(Op::infix(raised_operator, Right))
  };
}

fn token_to_span(pair: &Pair<Rule>) -> Span {
  let pest_span = pair.as_span();
  Span::new(pest_span.start(), pest_span.end())
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
    rule => unreachable!("Expression::parse expected binary operator, found {:?}", rule)
  }
}

fn parse_binary_expression(left: Expression, operator: Pair<Rule>, right: Expression) -> Expression {
  Expression {
    span: Span::new(left.span.start, right.span.end),
    kind: ExpressionKind::Binary {
      left: Box::new(left),
      operator: parse_binary_operator(operator),
      right: Box::new(right)
    }
  }
}

fn parse_expression(pairs: Pairs<Rule>) -> Expression {
  PRATT_PARSER
    .map_primary(|primary| match primary.as_rule() {
      Rule::let_block => {
        let span = token_to_span(&primary);
        let mut tokens = primary.into_inner();
        let value = Box::new(parse_expression(
          Pairs::single(tokens.next_back().unwrap())
        ));
        let assigments: Vec<Expression> = tokens.map(|assigment_token| {
          parse_expression(Pairs::single(assigment_token))
        }).collect();

        Expression {
          span,
          kind: ExpressionKind::LetBlock { assigments, value }
        }
      },
      Rule::if_else_block => {
        let span = token_to_span(&primary);
        let mut tokens = primary.into_inner();
        let test = Box::new(
          parse_expression(Pairs::single(tokens.next().unwrap()))
        );
        let consequent = Box::new(
          parse_expression(Pairs::single(tokens.next().unwrap()))
        );
        let alternative = Box::new(
          parse_expression(Pairs::single(tokens.next().unwrap()))
        );

        Expression {
          span,
          kind: ExpressionKind::IfElseBlock { test, consequent, alternative }
        }
      },
      Rule::match_block => {
        let span = token_to_span(&primary);
        let mut tokens = primary.into_inner();
        let value = Box::new(
          parse_expression(Pairs::single(tokens.next().unwrap()))
        );
        let has_default = tokens.len() % 2 != 0;
        let default: Option<Box<Expression>> =
          if has_default {
            Some(Box::new(
              parse_expression(Pairs::single(tokens.next_back().unwrap()))
            ))
          } else {
            None
          };
        
        let case_conditions = tokens
          .clone()
          .enumerate()
          .filter(|(index, _)| index % 2 == 0)
          .map(|(_, tk)| parse_expression(Pairs::single(tk)));
        let case_values = tokens
          .enumerate()
          .filter(|(index, _)| index % 2 != 0)
          .map(|(_, tk)| parse_expression(Pairs::single(tk)));

        let cases: Vec<(Expression, Expression)> = case_conditions.zip(case_values).collect();

        Expression {
          span,
          kind: ExpressionKind::MatchBlock {
            value,
            cases,
            default
          }
        }
      },
      Rule::perform_block => {
        let span = token_to_span(&primary);
        let sequence: Vec<Expression> =
          primary
          .into_inner()
          .map(|token| {
            parse_expression(Pairs::single(token))
          })
          .collect();

        Expression {
          span,
          kind: ExpressionKind::PerformBlock { sequence }
        }
      },
      Rule::linear_block => {
        let span = token_to_span(&primary);
        let expressions: Vec<Expression> =
          primary
          .into_inner()
          .map(|token| {
            parse_expression(Pairs::single(token))
          })
          .collect();

        Expression {
          span,
          kind: ExpressionKind::LinearBlock { expressions }
        }
      },
      Rule::call_expression => {
        let span = token_to_span(&primary);
        let mut call = primary.into_inner();
        let callee = Box::new(
          parse_expression(Pairs::single(call.next().unwrap()))
        );
        let params: Vec<Expression> = call.map(|expr| parse_expression(Pairs::single(expr))).collect();

        Expression {
          span,
          kind: ExpressionKind::Call {
            callee,
            params
          }
        }
      },
      Rule::lambda_expression => {
        let span = token_to_span(&primary);
        let mut tokens = primary.into_inner();
        let body = Box::new(parse_expression(Pairs::single(tokens.next_back().unwrap())));
        let params: Vec<String> = tokens.map(|identifier| identifier.as_str().to_string()).collect();


        Expression {
          span,
          kind: ExpressionKind::Lambda {
            params,
            body
          }
        }
      },
      Rule::partial_operation_left => {
        let span = token_to_span(&primary);
        let mut tokens = primary.into_inner();
        let operator = parse_binary_operator(tokens.next().unwrap());
        let right = Box::new(parse_expression(Pairs::single(tokens.next().unwrap())));

        Expression {
          span,
          kind: ExpressionKind::PartialLeftBinary { operator, right }
        }
      },
      Rule::partial_operation_right => {
        let span = token_to_span(&primary);
        let mut tokens = primary.into_inner();
        let left = Box::new(parse_expression(Pairs::single(tokens.next().unwrap())));
        let operator = parse_binary_operator(tokens.next().unwrap());

        Expression {
          span,
          kind: ExpressionKind::PartialRightBinary { left, operator }
        }
      },
      Rule::identifier => Expression {
        span: token_to_span(&primary),
        kind: ExpressionKind::Identifier(primary.as_str().to_string())
      },
      Rule::operator_identifier => Expression {
        span: token_to_span(&primary),
        kind: ExpressionKind::Identifier(
          primary
            .into_inner()
            .next()
            .unwrap()
            .as_str()
            .to_string()
        )
      },
      Rule::unknow_integer => Expression {
        span: token_to_span(&primary),
        kind: ExpressionKind::NumberLiteral {
          number: primary.as_str().to_string(),
          suffix: NumberSuffix::Unknow
        }
      },
      Rule::assigment_expression => {
        let span = token_to_span(&primary);
        let mut tokens = primary.into_inner();
        let identifier = tokens.next().unwrap().as_str().to_string();
        let value = Box::new(
          parse_expression(Pairs::single(tokens.next().unwrap()))
        );

        Expression {
          span,
          kind: ExpressionKind::Assigment { identifier, value } 
        }
      },
      Rule::expression => parse_expression(primary.into_inner()),
      rule => unreachable!("Expression::parse expected atom, found {:?}", rule)
    })
    .map_infix(|left, operator, right| parse_binary_expression(left, operator, right))
    .parse(pairs)
}

fn main() {
  let expression_str = args().nth(1).expect("1 argument required");
  println!("Source code: '{}'", expression_str);
  let tokens = QalcGrammar::parse(Rule::expression, &expression_str).expect("Parsing error");
  println!("AST: {:#?}", parse_expression(tokens));
}