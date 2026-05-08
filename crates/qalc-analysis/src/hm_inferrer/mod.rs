use qalc_ast::normalized_ast::NormalizedAST;
use qalc_ast::typed_ast::TypedAST;
use qalc_utils::{CompilerResult, CompilerStep, Diagnostic};

#[derive(Default)]
pub struct HMInferrer;

impl CompilerStep for HMInferrer {
  type Input = NormalizedAST;
  type Output = TypedAST;

  fn transform(&self, input: Self::Input, _: &mut Vec<Diagnostic>) -> CompilerResult<Self::Output> {
    todo!()
  }
}
