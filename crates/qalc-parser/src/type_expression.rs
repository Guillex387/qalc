use pest::iterators::Pair;
use qalc_ast::{ Span, concrete_ast::{ TypeExpression, TypeExpressionKind } };

use crate::{ Rule, shared::{ token_to_span, token_to_string } };

fn parse_type_identifier(token: Pair<Rule>) -> TypeExpressionKind {
  TypeExpressionKind::Identifier(token_to_string(token))
}

fn parse_type_slice(token: Pair<Rule>) -> TypeExpressionKind {
  let element_type_token = token.into_inner().next().unwrap();
  let element_type = parse_type_expression(element_type_token);

  TypeExpressionKind::Slice(Box::new(element_type))
}

fn parse_type_tuple(token: Pair<Rule>) -> TypeExpressionKind {
  let inner_types_tokens = token.into_inner();
  let inner_types: Vec<TypeExpression> = inner_types_tokens.map(parse_type_expression).collect();

  TypeExpressionKind::Tuple(inner_types)
}

fn parse_type_identifier_with_params(token: Pair<Rule>) -> TypeExpressionKind {
  let mut tokens = token.into_inner();
  let identifier = tokens.next().unwrap().as_str().to_owned();
  let params: Vec<TypeExpression> = tokens.map(parse_type_atom_expression).collect();

  TypeExpressionKind::IdentifierWithParams { identifier, params }
}

fn parse_type_atom_expression(token: Pair<Rule>) -> TypeExpression {
  let span = token_to_span(&token);
  let kind = match token.as_rule() {
    Rule::simple_identifier => parse_type_identifier(token),
    Rule::type_with_params => parse_type_identifier_with_params(token),
    Rule::type_slice => parse_type_slice(token),
    Rule::type_tuple => parse_type_tuple(token),
    Rule::type_inner => parse_type_expression(token.into_inner().next().unwrap()).kind,
    _ => unreachable!("Expected type expression rule, found {:?}", token),
  };

  TypeExpression { kind, span }
}

pub fn parse_type_expression(token: Pair<Rule>) -> TypeExpression {
  token
    .into_inner()
    .map(parse_type_atom_expression)
    .rev()
    .reduce(|result, input_type| TypeExpression {
      span: Span::new(input_type.span.start, result.span.end),
      kind: TypeExpressionKind::Transform {
        input_type: Box::new(input_type),
        output_type: Box::new(result),
      },
    })
    .unwrap()
}
