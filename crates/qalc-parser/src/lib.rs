mod declarations;
mod expression;
mod shared;
mod tokenizer;
mod type_expression;

// TODO: this is for testing, remove it when you integrate the feature
pub use tokenizer::*;

use std::{error::Error, fs};

use qalc_ast::concrete_ast::{ConcreteAST, Declaration};
use qalc_utils::CompilerResult;

use crate::declarations::parse_declaration;

pub fn parse_program(source_file: &str) -> Result<ConcreteAST, Box<dyn Error>> {
  let source_code = fs::read_to_string(source_file)?;
  let declarations_tokens = tokenize_source(source_file, &source_code)?;

  let declarations: Vec<Declaration> = declarations_tokens
    .map(parse_declaration)
    .collect::<CompilerResult<Vec<Declaration>>>()?;

  Ok(ConcreteAST { declarations })
}
