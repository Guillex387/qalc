use crate::{CompilerResult, Diagnostic};

pub trait CompilerStep {
  type Input;
  type Output;

  fn transform(
    &self,
    input: Self::Input,
    diagnostic: &mut Vec<Diagnostic>,
  ) -> CompilerResult<Self::Output>;
}
