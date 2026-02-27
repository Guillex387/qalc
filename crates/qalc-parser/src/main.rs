use std::env::args;

use pest::{Parser, iterators::{Pairs}, pratt_parser::PrattParser};
use pest_derive::Parser;
use qalc_ast::base_ast::{Expression, IntegerSuffix};

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

fn parse_binary_expression(left: Expression, operation: Rule, right: Expression) -> Expression {
  let operation = match operation {
    Rule::logic_or_operator => "||",
    Rule::logic_and_operator => "&&",
    Rule::or_operator => "|",
    Rule::and_operator => "&",
    Rule::xor_operator => "^",
    Rule::equal_operator => "==",
    Rule::not_equal_operator => "!=",
    Rule::less_equal_operator => "<=",
    Rule::greater_equal_operator => ">=",
    Rule::less_operator => "<",
    Rule::greater_operator => ">",
    Rule::add_operator => "+",
    Rule::subtract_operator => "-",
    Rule::multiply_operator => "*",
    Rule::divide_operator => "/",
    Rule::module_operator => "%",
    Rule::raised_operator => "**",
    rule => unreachable!("Expression::parse expected binary operator, found {:?}", rule)
  }.to_string();

  Expression::CallExpression { callee: Box::new(Expression::Identifier(operation)), params: vec![left, right] }
}

fn parse_expression(pairs: Pairs<Rule>) -> Expression {
  PRATT_PARSER
    .map_primary(|primary| match primary.as_rule() {
      Rule::let_block => {
        let mut tokens = primary.into_inner();
        let value = Box::new(parse_expression(
          Pairs::single(tokens.next_back().unwrap())
        ));
        let assigments: Vec<Expression> = tokens.map(|assigment_token| {
          parse_expression(Pairs::single(assigment_token))
        }).collect();

        Expression::LetBlock { assigments, value }
      },
      Rule::if_else_block => {
        let mut tokens = primary.into_inner();
        let test = tokens.next().unwrap();
        let consequent = tokens.next().unwrap();
        let alternative = tokens.next().unwrap();

        Expression::IfElseBlock {
          test: Box::new(parse_expression(Pairs::single(test))),
          consequent: Box::new(parse_expression(Pairs::single(consequent))),
          alternative: Box::new(parse_expression(Pairs::single(alternative)))
        }
      },
      Rule::call_expression => {
        let mut call = primary.into_inner();
        let callee = call.next().unwrap();

        Expression::CallExpression {
          callee: Box::new(parse_expression(Pairs::single(callee))),
          params: call.map(|expr| parse_expression(Pairs::single(expr))).collect()
        }
      },
      Rule::lambda_expression => {
        let mut tokens = primary.into_inner();
        let body = tokens.next_back().unwrap();

        Expression::LambdaExpression {
          params: tokens.map(|identifier| identifier.as_str().to_string()).collect(),
          body: Box::new(parse_expression(Pairs::single(body)))
        }
      },
      Rule::partial_operation_left => {
        let mut tokens = primary.into_inner();
        let operator = tokens.next().unwrap().as_str().to_string();
        let right = tokens.next().unwrap();

        Expression::LambdaExpression {
          params: vec!["a".to_string()],
          body: Box::new(Expression::CallExpression {
            callee: Box::new(Expression::Identifier(operator)),
            params: vec![Expression::Identifier("a".to_string()), parse_expression(Pairs::single(right))]
          })
        }
      },
      Rule::partial_operation_right => {
        let mut tokens = primary.into_inner();
        let left = tokens.next().unwrap();
        let operator = tokens.next().unwrap().as_str().to_string();

        Expression::CallExpression {
          callee: Box::new(Expression::Identifier(operator.to_string())),
          params: vec![parse_expression(Pairs::single(left))]
        }
      },
      Rule::identifier => {
        Expression::Identifier(primary.as_str().to_string())
      },
      Rule::operator_identifier => {
        Expression::Identifier(
          primary
            .into_inner()
            .next()
            .unwrap()
            .as_str()
            .to_string()
        )
      }
      Rule::unknow_integer => {
        Expression::IntegerLiteral {
          number: primary.as_str().to_string(),
          suffix: IntegerSuffix::Unknow
        }
      },
      Rule::assigment_expression => {
        let mut tokens = primary.into_inner();
        let identifier = tokens.next().unwrap().as_str().to_string();
        let value = Box::new(
          parse_expression(Pairs::single(tokens.next().unwrap()))
        );

        Expression::Assigment { identifier, value }
      },
      Rule::expression => parse_expression(primary.into_inner()),
      rule => unreachable!("Expression::parse expected atom, found {:?}", rule)
    })
    .map_infix(|left, operation, right| {
      match operation.as_rule() {
        Rule::pipe_operator => {
          Expression::CallExpression {
            callee: Box::new(right),
            params: vec![left]
          }
        },
        Rule::apply_operator => {
          Expression::CallExpression {
            callee: Box::new(left),
            params: vec![right]
          }
        },
        _ => parse_binary_expression(left, operation.as_rule(), right)
      }
    })
    .parse(pairs)
}

fn main() {
  let expression_str = args().nth(1).expect("1 argument required");
  println!("Source code: '{}'", expression_str);
  let tokens = QalcGrammar::parse(Rule::expression, &expression_str).expect("Parsing error");
  println!("AST: {:#?}", parse_expression(tokens));
}