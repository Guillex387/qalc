use qalc_ast::concrete_ast::ConcreteExpression;
use qalc_utils::CompilerResult;

use crate::expression::{atoms::parse_atom_expression, parse_expression};
use crate::tokenizer::TokensIter;

pub fn consume_expression(tokens: &mut TokensIter) -> CompilerResult<Box<ConcreteExpression>> {
  Ok(Box::new(parse_expression(tokens.eat())?))
}

pub fn consume_back_expression(tokens: &mut TokensIter) -> CompilerResult<Box<ConcreteExpression>> {
  Ok(Box::new(parse_expression(tokens.eat_back())?))
}

pub fn consume_atom(tokens: &mut TokensIter) -> CompilerResult<Box<ConcreteExpression>> {
  Ok(Box::new(parse_atom_expression(tokens.eat())?))
}
