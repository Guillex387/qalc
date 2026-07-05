use qalc_ast::Spanned;
use qalc_ast::base_ast::{Pattern, TypeExpression};
use qalc_ast::concrete_ast::{ConcreteDeclaration, ConcreteDeclarationKind};
use qalc_utils::{CompilerResult, Diagnostic};

use crate::expression::parse_expression;
use crate::pattern::parse_pattern;
use crate::tokenizer::{Rule, Token};
use crate::type_expression::parse_type_expression;

/// Extrae el nombre de un token `identifier` (regla silenciosa que expone
/// directamente `simple_identifier`, cuyo texto ya es el nombre, o bien
/// `operator_identifier`, cuyo texto incluye las comillas invertidas que hay
/// que descartar para quedarse solo con el operador, por ejemplo `` `+` ``).
fn parse_identifier_token(token: Token) -> String {
  match token.rule() {
    Rule::operator_identifier => token.inner().eat().to_string(),
    _ => token.to_string(),
  }
}

fn parse_function_annotation(token: Token) -> (String, TypeExpression<Spanned>) {
  let mut tokens = token.inner();
  let identifier = parse_identifier_token(tokens.eat());
  let type_expression = parse_type_expression(tokens.eat());

  (identifier, type_expression)
}

fn parse_type_declaration(token: Token) -> ConcreteDeclarationKind {
  let mut tokens = token.inner();
  let identifier = parse_identifier_token(tokens.eat());
  let mut attributes: Vec<(String, TypeExpression<Spanned>)> = Vec::new();
  let mut linear = false;

  for token in tokens {
    match token.rule() {
      Rule::type_function_annotation => attributes.push(parse_function_annotation(token)),
      Rule::type_restriction => {
        linear = true;
      }
      _ => unreachable!(),
    }
  }

  ConcreteDeclarationKind::Type {
    identifier,
    attributes,
    linear,
  }
}

fn parse_function_declaration(token: Token) -> CompilerResult<ConcreteDeclarationKind> {
  let mut tokens = token.inner();
  let (identifier, type_annotation) = parse_function_annotation(tokens.eat());

  let function_identifier_token = tokens.eat();
  let function_identifier_span = function_identifier_token.span();
  let function_identifier = parse_identifier_token(function_identifier_token);
  if identifier != function_identifier {
    return Err(Diagnostic::error(
      "Definition name mismatch",
      format!("expected '{}'", identifier),
      function_identifier_span,
    ));
  }

  let body = parse_expression(tokens.eat_back())?;
  let params: Vec<Pattern<Spanned>> = tokens.map(parse_pattern).collect();

  Ok(ConcreteDeclarationKind::Function {
    identifier,
    type_annotation,
    params,
    body,
  })
}

pub fn parse_declaration(token: Token) -> CompilerResult<ConcreteDeclaration> {
  let span = token.span();
  let kind = match token.rule() {
    Rule::function_declaration => parse_function_declaration(token)?,
    Rule::type_declaration => parse_type_declaration(token),
    _ => unreachable!("Expected declaration, found: {:?}", token),
  };

  Ok(ConcreteDeclaration { kind, span })
}
