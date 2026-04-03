use pest::{
  Parser,
  iterators::{Pair, Pairs},
};
use pest_derive::Parser;
use qalc_utils::{CompilerResult, Span};

use crate::shared::pest_error_to_diagnostic;

#[derive(Debug, Clone)]
pub struct Token<'a> {
  token: Pair<'a, Rule>,
  source_file: &'a str,
}

impl<'a> Token<'a> {
  pub fn new(token: Pair<'a, Rule>, source_file: &'a str) -> Token<'a> {
    Token { token, source_file }
  }

  pub fn source_file(&self) -> &'a str {
    self.source_file
  }

  pub fn rule(&self) -> Rule {
    self.token.as_rule()
  }

  pub fn span(&self) -> Span {
    let pest_span = self.token.as_span();
    Span::new(self.source_file, pest_span.start(), pest_span.end())
  }

  pub fn inner(self) -> TokensIter<'a> {
    TokensIter::new(self.token.into_inner(), self.source_file)
  }
}

impl ToString for Token<'_> {
  fn to_string(&self) -> String {
    self.token.as_str().to_owned()
  }
}

impl<'a> Into<Pair<'a, Rule>> for Token<'a> {
  fn into(self) -> Pair<'a, Rule> {
    self.token
  }
}

#[derive(Debug)]
pub struct TokensIter<'a> {
  tokens: Pairs<'a, Rule>,
  source_file: &'a str,
}

impl<'a> TokensIter<'a> {
  fn new(tokens: Pairs<'a, Rule>, source_file: &'a str) -> Self {
    TokensIter {
      tokens,
      source_file,
    }
  }

  pub fn eat(&mut self) -> Token<'a> {
    self.next().unwrap()
  }

  pub fn eat_back(&mut self) -> Token<'a> {
    self.next_back().unwrap()
  }
}

impl<'a> Iterator for TokensIter<'a> {
  type Item = Token<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    self
      .tokens
      .next()
      .map(|raw_token| Token::new(raw_token, self.source_file))
  }
}

impl DoubleEndedIterator for TokensIter<'_> {
  fn next_back(&mut self) -> Option<Self::Item> {
    self
      .tokens
      .next_back()
      .map(|raw_token| Token::new(raw_token, self.source_file))
  }
}

impl ExactSizeIterator for TokensIter<'_> {
  #[inline]
  fn len(&self) -> usize {
    self.tokens.len()
  }
}

impl<'a> Into<Pairs<'a, Rule>> for TokensIter<'a> {
  fn into(self) -> Pairs<'a, Rule> {
    self.tokens
  }
}

#[allow(dead_code)]
#[derive(Parser)]
#[grammar = "grammar.pest"]
struct QalcGrammar;

pub fn tokenize_source<'a>(
  source_file: &'a str,
  source_code: &'a str,
) -> CompilerResult<TokensIter<'a>> {
  let mut program_tokens = TokensIter::new(
    QalcGrammar::parse(Rule::program, source_code)
      .map_err(|error| pest_error_to_diagnostic(source_file, error))?,
    source_file,
  );
  let mut declarations_tokens = program_tokens.eat().inner();
  declarations_tokens.eat_back(); // CONSUME EOF token

  Ok(declarations_tokens)
}
