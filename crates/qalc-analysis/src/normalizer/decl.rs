use qalc_ast::concrete_ast::{
  Declaration as CDecl, DeclarationKind as CDeclKind, Expression as CExpr, Pattern, TypeExpression,
};
use qalc_ast::normalized_ast::{Declaration as NDecl, DeclarationKind as NDeclKind};
use qalc_utils::CompilerResult;

use crate::normalizer::{desugar::transform_params_pattern_to_let, expr::transform_expression};

fn adapt_type_decl(
  identifier: String,
  attributes: Vec<(String, TypeExpression)>,
  linear: bool,
) -> NDeclKind {
  NDeclKind::Type {
    identifier,
    attributes,
    linear,
  }
}

fn transform_function_decl(
  identifier: String,
  type_annotation: TypeExpression,
  params: Vec<Pattern>,
  body: CExpr,
) -> CompilerResult<NDeclKind> {
  if params.is_empty() {
    return Ok(NDeclKind::Function {
      identifier,
      type_annotation,
      params: Vec::new(),
      body: transform_expression(body)?,
    });
  }

  let (params, body) = transform_params_pattern_to_let(params, body)?;

  Ok(NDeclKind::Function {
    identifier,
    type_annotation,
    params,
    body,
  })
}

pub fn transform_declaration(declaration: CDecl) -> CompilerResult<NDecl> {
  let span = declaration.span;
  let kind = match declaration.kind {
    CDeclKind::Type {
      identifier,
      attributes,
      linear,
    } => adapt_type_decl(identifier, attributes, linear),
    CDeclKind::Function {
      identifier,
      type_annotation,
      params,
      body,
    } => transform_function_decl(identifier, type_annotation, params, body)?,
  };

  Ok(NDecl { kind, span })
}
