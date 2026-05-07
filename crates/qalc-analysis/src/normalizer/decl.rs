use qalc_ast::Spanned;
use qalc_ast::base_ast::{Declaration, DeclarationKind, Pattern, TypeExpression};
use qalc_ast::concrete_ast::{ConcreteDeclaration, ConcreteDeclarationKind, ConcreteExpression};
use qalc_utils::CompilerResult;

use crate::normalizer::{desugar::transform_params_pattern_to_let, expr::transform_expression};

fn adapt_type_decl(
  identifier: String,
  attributes: Vec<(String, TypeExpression<Spanned>)>,
  linear: bool,
) -> DeclarationKind<Spanned> {
  DeclarationKind::Type {
    identifier,
    attributes,
    linear,
  }
}

fn transform_function_decl(
  identifier: String,
  type_annotation: TypeExpression<Spanned>,
  params: Vec<Pattern<Spanned>>,
  body: ConcreteExpression,
) -> CompilerResult<DeclarationKind<Spanned>> {
  if params.is_empty() {
    return Ok(DeclarationKind::Function {
      identifier,
      type_annotation,
      params: Vec::new(),
      body: transform_expression(body)?,
    });
  }

  let (params, body) = transform_params_pattern_to_let(params, body)?;

  Ok(DeclarationKind::Function {
    identifier,
    type_annotation,
    params,
    body,
  })
}

pub fn transform_declaration(
  declaration: ConcreteDeclaration,
) -> CompilerResult<Declaration<Spanned>> {
  let span = declaration.span;
  let kind = match declaration.kind {
    ConcreteDeclarationKind::Type {
      identifier,
      attributes,
      linear,
    } => adapt_type_decl(identifier, attributes, linear),
    ConcreteDeclarationKind::Function {
      identifier,
      type_annotation,
      params,
      body,
    } => transform_function_decl(identifier, type_annotation, params, body)?,
  };

  Ok(Declaration { kind, meta: span })
}
