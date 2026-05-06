use qalc_ast::{ASTTransform, concrete_ast::ConcreteAST, normalized_ast::NormalizedAST};
use qalc_utils::{CompilerResult, Diagnostic};

use crate::normalizer::decl::transform_declaration;

mod adapters;
mod decl;
mod desugar;
mod expr;

#[derive(Default)]
pub struct Normalizer;

impl ASTTransform for Normalizer {
  type Input = ConcreteAST;
  type Output = NormalizedAST;

  fn transform(&self, input: Self::Input, _: &mut Vec<Diagnostic>) -> CompilerResult<Self::Output> {
    Ok(NormalizedAST {
      declarations: input
        .declarations
        .into_iter()
        .map(transform_declaration)
        .collect::<CompilerResult<_>>()?,
    })
  }
}
