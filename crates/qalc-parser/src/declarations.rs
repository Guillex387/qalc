use qalc_ast::concrete_ast::{Declaration, DeclarationKind, TypeExpression};
use qalc_utils::{CompilerResult, Diagnostic};

use crate::expression::parse_expression;
use crate::tokenizer::{Rule, Token};
use crate::type_expression::parse_type_expression;

fn parse_function_annotation(token: Token) -> (String, TypeExpression) {
  let mut tokens = token.inner();
  let identifier = tokens.eat().to_string();
  let type_expression = parse_type_expression(tokens.eat());

  (identifier, type_expression)
}

fn parse_type_declaration(token: Token) -> DeclarationKind {
  let mut tokens = token.inner();
  let identifier = tokens.eat().to_string();
  let mut attributes: Vec<(String, TypeExpression)> = Vec::new();
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

  DeclarationKind::Type {
    identifier,
    attributes,
    linear,
  }
}

fn parse_function_declaration(token: Token) -> CompilerResult<DeclarationKind> {
  let mut tokens = token.inner();
  let (identifier, type_annotation) = parse_function_annotation(tokens.eat());

  let function_identifier_token = tokens.eat();
  let function_identifier = function_identifier_token.to_string();
  if identifier != function_identifier {
    return Err(Diagnostic::error(
      "Definition name mismatch",
      format!("expected '{}'", identifier),
      function_identifier_token.span(),
    ));
  }

  let body = parse_expression(tokens.eat_back());
  let params: Vec<String> = tokens.map(|token| token.to_string()).collect();

  Ok(DeclarationKind::Function {
    identifier,
    type_annotation,
    params,
    body,
  })
}

pub fn parse_declaration(token: Token) -> CompilerResult<Declaration> {
  let span = token.span();
  let kind = match token.rule() {
    Rule::function_declaration => parse_function_declaration(token)?,
    Rule::type_declaration => parse_type_declaration(token),
    _ => unreachable!("Expected declaration, found: {:?}", token),
  };

  Ok(Declaration { kind, span })
}
