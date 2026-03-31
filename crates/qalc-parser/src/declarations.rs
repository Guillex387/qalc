use pest::iterators::Pair;
use qalc_ast::concrete_ast::{ Declaration, DeclarationKind, TypeExpression };

use crate::{
  Rule,
  expression::parse_expression,
  shared::{ token_to_span, token_to_string },
  type_expression::parse_type_expression,
};

fn parse_function_annotation(token: Pair<Rule>) -> (String, TypeExpression) {
  let mut tokens = token.into_inner();
  let identifier = tokens.next().unwrap().as_str().to_owned();
  let type_expression = parse_type_expression(tokens.next().unwrap());

  (identifier, type_expression)
}

fn parse_type_declaration(token: Pair<Rule>) -> DeclarationKind {
  let mut tokens = token.into_inner();
  let identifier = tokens.next().unwrap().as_str().to_owned();
  let mut attributes: Vec<(String, TypeExpression)> = Vec::new();
  let mut linear = false;

  for token in tokens {
    match token.as_rule() {
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

fn parse_function_declaration(token: Pair<Rule>) -> DeclarationKind {
  let mut tokens = token.into_inner();
  let (identifier, type_annotation) = parse_function_annotation(tokens.next().unwrap());
  let function_identifier = tokens.next().unwrap().as_str().to_owned();
  // TODO: convert this to a standard error
  assert_eq!(identifier, function_identifier);
  let body = parse_expression(tokens.next_back().unwrap());
  let params: Vec<String> = tokens.map(token_to_string).collect();

  DeclarationKind::Function {
    identifier,
    type_annotation,
    params,
    body,
  }
}

pub fn parse_declaration(token: Pair<Rule>) -> Declaration {
  let span = token_to_span(&token);
  let kind = match token.as_rule() {
    Rule::function_declaration => parse_function_declaration(token),
    Rule::type_declaration => parse_type_declaration(token),
    _ => unreachable!("Expected declaration, found: {:?}", token),
  };

  Declaration { kind, span }
}
