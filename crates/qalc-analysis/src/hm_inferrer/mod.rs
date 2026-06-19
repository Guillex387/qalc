mod apply;
mod infer;
mod unify;

use std::rc::Rc;

use qalc_ast::base_ast::DeclarationKind;
use qalc_ast::normalized_ast::NormalizedAST;
use qalc_ast::scope::{Scope, SymbolInfo, SymbolKind};
use qalc_ast::typed_ast::TypedAST;
use qalc_builtins::BuiltinDef;
use qalc_utils::{CompilerResult, CompilerStep, Diagnostic, Source, Span};

use apply::finalize_decl;
use infer::{infer_decl, type_expr_to_infer_type};
use unify::unify;

pub struct HMInferrer {
  builtins: Vec<BuiltinDef>,
}

impl HMInferrer {
  pub fn new(builtins: Vec<BuiltinDef>) -> Self {
    Self { builtins }
  }

  pub fn with_builtins(mut self, builtins: Vec<BuiltinDef>) -> Self {
    self.builtins = builtins;
    self
  }
}

impl Default for HMInferrer {
  fn default() -> Self {
    Self { builtins: vec![] }
  }
}

impl CompilerStep for HMInferrer {
  type Input = NormalizedAST;
  type Output = TypedAST;

  fn transform(&self, input: NormalizedAST, _: &mut Vec<Diagnostic>) -> CompilerResult<TypedAST> {
    let global_scope = Rc::new(Scope::new(None));

    if !self.builtins.is_empty() {
      let builtin_source = Rc::new(Source::memory(""));
      let builtin_span = Span::new(&builtin_source, 0, 0);
      for builtin in &self.builtins {
        global_scope.define(SymbolInfo {
          name: builtin.name.clone(),
          kind: SymbolKind::Function,
          infer_type: builtin.ttype.clone(),
          span: builtin_span.clone(),
        });
      }
    }

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

    let substitutions = unify(all_constraints)?;

    let typed_decls = inferred_decls
      .into_iter()
      .map(|decl| finalize_decl(decl, &substitutions))
      .collect::<Result<Vec<_>, _>>()?;

    Ok(TypedAST {
      declarations: typed_decls,
      global_scope,
    })
  }
}
