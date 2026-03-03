mod expression;
mod shared;

use std::env::args;

use pest::Parser;
use pest_derive::Parser;

use crate::expression::parse_expression_pairs;

#[allow(dead_code)]
#[derive(Parser)]
#[grammar = "grammar.pest"]
struct QalcGrammar;

fn main() {
  let expression_str = args().nth(1).expect("1 argument required");
  println!("Source code: '{}'", expression_str);
  let tokens = QalcGrammar::parse(Rule::expression, &expression_str).expect("Parsing error");
  println!("AST: {:#?}", parse_expression_pairs(tokens));
}
