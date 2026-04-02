mod expression;
mod declarations;
mod shared;
mod tokenizer;
mod type_expression;

// TODO: this is for testing, remove it when you integrate the feature
pub use tokenizer::*;

use std::{ error::Error, fs };

use pest::Parser;
use pest_derive::Parser;
use qalc_ast::concrete_ast::{ ConcreteAST, Declaration };
use qalc_utils::CompilerResult;

use crate::{ declarations::parse_declaration, shared::pest_error_to_diagnostic };

#[allow(dead_code)]
#[derive(Parser)]
#[grammar = "grammar.pest"]
struct QalcGrammar;

pub fn parse_program(filename: &str) -> Result<ConcreteAST, Box<dyn Error>> {
  let source = fs::read_to_string(filename)?;
  let mut program_tokens = QalcGrammar::parse(Rule::program, &source).map_err(|error|
    pest_error_to_diagnostic(filename, error)
  )?;
  let mut declarations_tokens = program_tokens.next().unwrap().into_inner();
  declarations_tokens.next_back().unwrap(); // CONSUME EOF token

  let declarations: Vec<Declaration> = declarations_tokens
    .map(|token| parse_declaration(filename, token))
    .collect::<CompilerResult<Vec<Declaration>>>()?;

  Ok(ConcreteAST { declarations })
}
