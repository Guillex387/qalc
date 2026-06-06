mod infer;

use std::rc::Rc;

use qalc_ast::Spanned;
use qalc_ast::base_ast::{ExpressionKind, FloatSuffix, NumberSuffix};
use qalc_ast::scope::Scope;
use qalc_ast::typed_ast::{TypeWithUnknow, Typed, TypedAST};
use qalc_ast::{base_ast::Expression, normalized_ast::NormalizedAST};
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
