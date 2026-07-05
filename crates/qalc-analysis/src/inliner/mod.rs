use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use qalc_ast::base_ast::{
  Declaration, DeclarationKind, Expression, TypeExpression, TypeExpressionKind,
};
use qalc_ast::flat_ast::{FlatAST, FlatParam, FlatResult};
use qalc_ast::typed_ast::{Type, Typed, TypedAST};
use qalc_utils::{CompilerResult, CompilerStep, Diagnostic, Source, Span};

use expr::inline_expr;

mod expr;

pub(crate) struct FnDef {
  pub params: Vec<String>,
  pub body: Expression<Typed>,
  pub ttype: Type,
  pub annotation: TypeExpression<Typed>,
}

#[derive(Default)]
pub struct Inliner;

impl CompilerStep for Inliner {
  type Input = TypedAST;
  type Output = FlatAST;

  fn transform(&self, input: TypedAST, _: &mut Vec<Diagnostic>) -> CompilerResult<FlatAST> {
    let TypedAST {
      declarations,
      global_scope,
    } = input;

    let fallback_span = declarations
      .first()
      .map(|d| d.meta.span.clone())
      .unwrap_or_else(|| Span::new(&Rc::new(Source::memory(String::new())), 0, 0));

    let mut funcs: HashMap<String, FnDef> = HashMap::new();
    for decl in declarations {
      let Declaration { kind, meta } = decl;
      if let DeclarationKind::Function {
        identifier,
        type_annotation,
        params,
        body,
      } = kind
      {
        funcs.insert(
          identifier,
          FnDef {
            params,
            body,
            ttype: meta.ttype,
            annotation: type_annotation,
          },
        );
      }
    }

    let main = funcs.get("main").ok_or_else(|| {
      Diagnostic::error(
        "no se encontró la función 'main'",
        "el aplanado necesita una función de entrada llamada 'main'",
        fallback_span.clone(),
      )
    })?;

    let main_params = main.params.clone();
    let main_ttype = main.ttype.clone();
    let main_annotation = main.annotation.clone();
    let main_body = main.body.clone();

    let mut params = Vec::new();
    let mut cursor = &main_ttype;
    for name in &main_params {
      if let Type::Arrow(from, to) = cursor {
        params.push(FlatParam {
          name: name.clone(),
          ttype: (**from).clone(),
        });
        cursor = to;
      } else {
        return Err(Diagnostic::error(
          "el tipo de 'main' no concuerda con su número de parámetros",
          "no se pudo pelar el tipo función",
          fallback_span,
        ));
      }
    }
    let result_ty = cursor.clone();
    let result = flat_result_from_type(&result_ty, &main_annotation);

    let shadowed: HashSet<String> = main_params.iter().cloned().collect();
    let body = inline_expr(main_body, &funcs, &shadowed, &HashMap::new())?;

    Ok(FlatAST {
      params,
      body,
      result,
      global_scope,
    })
  }
}

fn flat_result_from_type(result_ty: &Type, annotation: &TypeExpression<Typed>) -> FlatResult {
  let (qio, inner) = match result_ty {
    Type::Builtin { id, params, .. } if id == "qio" => (true, params.first()),
    _ => (false, None),
  };
  let qio = qio || annotation_output_is_qio(annotation);
  let effective = inner.unwrap_or(result_ty);
  let arity = match effective {
    Type::Tuple(ts) => ts.len(),
    _ => 1,
  };
  FlatResult { arity, qio }
}

fn annotation_output_is_qio(te: &TypeExpression<Typed>) -> bool {
  let mut cur = te;
  while let TypeExpressionKind::Transform { output_type, .. } = &cur.kind {
    cur = output_type;
  }
  matches!(
    &cur.kind,
    TypeExpressionKind::Identifier(id)
      | TypeExpressionKind::IdentifierWithParams { identifier: id, .. }
    if id == "qio"
  )
}
