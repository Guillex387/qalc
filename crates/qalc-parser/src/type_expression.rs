use qalc_ast::{
  Spanned,
  base_ast::{TypeExpression, TypeExpressionKind},
};

use crate::tokenizer::{Rule, Token};

fn parse_type_identifier(token: Token) -> TypeExpressionKind<Spanned> {
  TypeExpressionKind::Identifier(token.to_string())
}

fn parse_type_slice(token: Token) -> TypeExpressionKind<Spanned> {
  let element_type = parse_type_expression(token.inner().eat());

  TypeExpressionKind::Slice(Box::new(element_type))
}

fn parse_type_tuple(token: Token) -> TypeExpressionKind<Spanned> {
  let inner_types_tokens = token.inner();
  let inner_types: Vec<TypeExpression<Spanned>> =
    inner_types_tokens.map(parse_type_expression).collect();

  TypeExpressionKind::Tuple(inner_types)
}

fn parse_type_identifier_with_params(token: Token) -> TypeExpressionKind<Spanned> {
  let mut tokens = token.inner();
  let identifier = tokens.eat().to_string();
  let params: Vec<TypeExpression<Spanned>> = tokens.map(parse_type_atom_expression).collect();

  TypeExpressionKind::IdentifierWithParams { identifier, params }
}

fn parse_type_atom_expression(token: Token) -> TypeExpression<Spanned> {
  let span = token.span();
  let kind = match token.rule() {
    Rule::simple_identifier => parse_type_identifier(token),
    Rule::type_with_params => parse_type_identifier_with_params(token),
    Rule::type_slice => parse_type_slice(token),
    Rule::type_tuple => parse_type_tuple(token),
    Rule::type_inner => parse_type_expression(token.inner().eat()).kind,
    _ => unreachable!("Expected type expression rule, found {:?}", token),
  };

  TypeExpression { kind, meta: span }
}

pub fn parse_type_expression(token: Token) -> TypeExpression<Spanned> {
  token
    .inner()
    .map(parse_type_atom_expression)
    .rev()
    .reduce(|result, input_type| TypeExpression {
      meta: input_type.meta.join(&result.meta),
      kind: TypeExpressionKind::Transform {
        input_type: Box::new(input_type),
        output_type: Box::new(result),
      },
    })
    .unwrap()
}
