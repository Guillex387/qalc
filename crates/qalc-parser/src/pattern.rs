use qalc_ast::concrete_ast::Pattern;

use crate::tokenizer::{Rule, Token};

pub fn parse_pattern(token: Token) -> Pattern {
  let mut tokens = token.inner().peekable();
  let first_token = tokens.peek().unwrap();

  match first_token.rule() {
    Rule::simple_identifier => {
      let identifier = first_token.to_string();
      if identifier == "_" {
        Pattern::Wildcard
      } else {
        Pattern::Variable(identifier)
      }
    }
    Rule::pattern => Pattern::Tuple(tokens.map(parse_pattern).collect()),
    _ => unreachable!("Expected pattern, found {:?}", first_token.rule()),
  }
}
