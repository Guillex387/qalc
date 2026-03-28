mod declarations;
mod expression;
mod shared;
mod type_expression;

use std::env::args;

use pest::Parser;
use pest_derive::Parser;
use qalc_ast::concrete_ast::{ ConcreteAST, Declaration };

use crate::{ declarations::parse_declaration, expression::parse_expression_pairs };

#[allow(dead_code)]
#[derive(Parser)]
#[grammar = "grammar.pest"]
struct QalcGrammar;

pub fn parse_program(source: &str) -> ConcreteAST {
  // TODO: make standard error handling in this sections
  let declaration_tokens = QalcGrammar::parse(Rule::program, source).expect("Parsing error");
  let declarations: Vec<Declaration> = declaration_tokens.map(parse_declaration).collect();

  ConcreteAST { declarations }
}

fn main() {
  // TODO: delete main fn, and transform the parser in a library
  let expression_str = args().nth(1).expect("1 argument required");
  println!("Source code: '{}'", expression_str);
  let tokens = QalcGrammar::parse(Rule::expression, &expression_str).expect("Parsing error");
  println!("AST: {:#?}", parse_expression_pairs(tokens));
}
