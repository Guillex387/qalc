mod expression;
mod declarations;
mod shared;
mod type_expression;

use std::{ env::args, fs };

use pest::Parser;
use pest_derive::Parser;
use qalc_ast::concrete_ast::{ ConcreteAST, Declaration };

use crate::declarations::parse_declaration;

#[allow(dead_code)]
#[derive(Parser)]
#[grammar = "grammar.pest"]
struct QalcGrammar;

pub fn parse_program(source: &str) -> ConcreteAST {
  // TODO: make standard error handling in this sections
  let mut program_tokens = QalcGrammar::parse(Rule::program, source).expect("Parsing error");
  let mut declarations_tokens = program_tokens.next().unwrap().into_inner();
  declarations_tokens.next_back().unwrap(); // CONSUME EOF token
  let declarations: Vec<Declaration> = declarations_tokens.map(parse_declaration).collect();

  ConcreteAST { declarations }
}

fn main() {
  // TODO: delete main fn, and transform the parser in a library
  let file_name = args().nth(1).expect("1 argument required");
  let source = fs::read_to_string(file_name).expect("Error reading source file");
  let concrete_ast = parse_program(&source);
  println!("AST: {:#?}", concrete_ast);
}
