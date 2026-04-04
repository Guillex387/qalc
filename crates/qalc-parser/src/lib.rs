mod declarations;
mod expression;
mod pattern;
mod shared;
mod tokenizer;
mod type_expression;

use std::rc::Rc;

use qalc_ast::concrete_ast::{ConcreteAST, Declaration};
use qalc_utils::{CompilerResult, Source};

use crate::{declarations::parse_declaration, tokenizer::tokenize};

pub fn parse_program(source: &Rc<Source>) -> CompilerResult<ConcreteAST> {
  let declarations_tokens = tokenize(source)?;

  let declarations: Vec<Declaration> = declarations_tokens
    .map(parse_declaration)
    .collect::<CompilerResult<Vec<Declaration>>>()?;

  Ok(ConcreteAST { declarations })
}
