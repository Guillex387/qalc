use qalc_utils::{CompilerResult, Diagnostic};

pub trait ASTTransform {
  type Input;
  type Output;

  fn transform(
    &self,
    input: Self::Input,
    diagnostic: &mut Vec<Diagnostic>,
  ) -> CompilerResult<Self::Output>;
}
