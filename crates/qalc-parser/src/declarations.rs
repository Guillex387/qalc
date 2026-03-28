use pest::iterators::Pair;
use qalc_ast::concrete_ast::{ Declaration, DeclarationKind };

use crate::{ Rule, shared::token_to_span };

fn parse_type_declaration(token: Pair<Rule>) -> DeclarationKind {
  todo!()
}

fn parse_function_declaration(token: Pair<Rule>) -> DeclarationKind {
  todo!()
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
