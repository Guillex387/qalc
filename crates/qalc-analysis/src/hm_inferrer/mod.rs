mod infer;

use std::rc::Rc;

use qalc_ast::base_ast::DeclarationKind;
use qalc_ast::normalized_ast::NormalizedAST;
use qalc_ast::scope::{Scope, SymbolInfo, SymbolKind};
use qalc_ast::typed_ast::TypedAST;
use qalc_utils::{CompilerResult, CompilerStep, Diagnostic};

use infer::{infer_decl, type_expr_to_infer_type};

#[derive(Default)]
pub struct HMInferrer;

impl CompilerStep for HMInferrer {
  type Input = NormalizedAST;
  type Output = TypedAST;

  fn transform(&self, input: NormalizedAST, _: &mut Vec<Diagnostic>) -> CompilerResult<TypedAST> {
    let global_scope = Rc::new(Scope::new(None));
    let mut declared_types = Vec::new();

    for decl in &input.declarations {
      match &decl.kind {
        DeclarationKind::Function {
          identifier,
          type_annotation,
          ..
        } => {
          let decl_type = type_expr_to_infer_type(type_annotation.clone());
          global_scope.define_decl(SymbolInfo {
            name: identifier.clone(),
            kind: SymbolKind::Function,
            infer_type: decl_type.clone(),
            span: decl.meta.clone(),
          });
          declared_types.push(decl_type);
        }
        DeclarationKind::Type { .. } => {
          return Err(Diagnostic::error(
            "Type declarations are not supported in the MVP.",
            "Feature outside MVP — only function declarations are supported.",
            decl.meta.clone(),
          ));
        }
      }
    }

    let mut all_constraints = Vec::new();
    let mut inferred_decls = Vec::new();
    for (decl, declared_type) in input.declarations.into_iter().zip(declared_types) {
      let (inferred_decl, constraints) = infer_decl(decl, declared_type, &global_scope)?;
      all_constraints.extend(constraints);
      inferred_decls.push(inferred_decl);
    }

    todo!("Solve constraints and produce TypedAST")
  }
}
