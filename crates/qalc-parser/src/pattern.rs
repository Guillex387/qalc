use qalc_ast::concrete_ast::{Pattern, PatternKind};

use crate::tokenizer::{Rule, Token};

pub fn parse_pattern(token: Token) -> Pattern {
  let span = token.span();
  let mut tokens = token.inner().peekable();
  let first_token = tokens.peek().unwrap();

  let kind = match first_token.rule() {
    Rule::simple_identifier => {
      let identifier = first_token.to_string();
      if identifier == "_" {
        PatternKind::Wildcard
      } else {
        PatternKind::Variable(identifier)
      }
    }
    Rule::pattern => PatternKind::Tuple(tokens.map(parse_pattern).collect()),
    _ => unreachable!("Expected pattern, found {:?}", first_token.rule()),
  };

  Pattern { kind, span }
}
