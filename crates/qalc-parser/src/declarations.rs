use pest::iterators::Pair;
use qalc_ast::concrete_ast::{ Declaration, DeclarationKind, TypeExpression };
use qalc_utils::{ CompilerResult, Diagnostic };

use crate::{
  Rule,
  expression::parse_expression,
  shared::{ token_to_span, token_to_string },
  type_expression::parse_type_expression,
};

fn parse_function_annotation(
  filename: &str,
  token: Pair<Rule>
) -> (String, TypeExpression) {
  let mut tokens = token.into_inner();
  let identifier = tokens.next().unwrap().as_str().to_owned();
  let type_expression = parse_type_expression(filename, tokens.next().unwrap());

  (identifier, type_expression)
}

fn parse_type_declaration(filename: &str, token: Pair<Rule>) -> DeclarationKind {
  let mut tokens = token.into_inner();
  let identifier = tokens.next().unwrap().as_str().to_owned();
  let mut attributes: Vec<(String, TypeExpression)> = Vec::new();
  let mut linear = false;

  for token in tokens {
    match token.as_rule() {
      Rule::type_function_annotation =>
        attributes.push(parse_function_annotation(filename, token)),
      Rule::type_restriction => {
        linear = true;
      }
      _ => unreachable!(),
    }
  }

  DeclarationKind::Type {
    identifier,
    attributes,
    linear,
  }
}

fn parse_function_declaration(
  filename: &str,
  token: Pair<Rule>
) -> CompilerResult<DeclarationKind> {
  let mut tokens = token.into_inner();
  let (identifier, type_annotation) = parse_function_annotation(
    filename,
    tokens.next().unwrap()
  );
  let function_identifier_token = tokens.next().unwrap();
  let function_identifier = function_identifier_token.as_str().to_owned();
  if identifier != function_identifier {
    let error_span = token_to_span(filename, &function_identifier_token);
    return Err(
      Diagnostic::error(
        "Definition name mismatch",
        format!("expected '{}'", identifier),
        error_span
      )
    );
  }

  let body = parse_expression(filename, tokens.next_back().unwrap());
  let params: Vec<String> = tokens.map(token_to_string).collect();

  Ok(DeclarationKind::Function {
    identifier,
    type_annotation,
    params,
    body,
  })
}

pub fn parse_declaration(
  filename: &str,
  token: Pair<Rule>
) -> CompilerResult<Declaration> {
  let span = token_to_span(filename, &token);
  let kind = match token.as_rule() {
    Rule::function_declaration => parse_function_declaration(filename, token)?,
    Rule::type_declaration => parse_type_declaration(filename, token),
    _ => unreachable!("Expected declaration, found: {:?}", token),
  };

  Ok(Declaration { kind, span })
}
