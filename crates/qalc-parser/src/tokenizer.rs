use pest::iterators::{ Pair, Pairs };
use qalc_utils::Span;

use crate::Rule;

#[derive(Debug, Clone)]
pub struct Token<'a> {
  token: Pair<'a, Rule>,
  source_file: &'a str,
}

impl<'a> Token<'a> {
  pub fn new(token: Pair<'a, Rule>, source_file: &'a str) -> Token<'a> {
    Token { token, source_file }
  }

  pub fn span(&self) -> Span {
    let pest_span = self.token.as_span();
    Span::new(self.source_file, pest_span.start(), pest_span.end())
  }

  pub fn inner(self) -> TokensIter<'a> {
    TokensIter::new(self.token.into_inner(), self.source_file)
  }
}

impl<'a> ToString for Token<'a> {
  fn to_string(&self) -> String {
    self.token.as_str().to_owned()
  }
}

impl<'a> Into<Pair<'a, Rule>> for Token<'a> {
  fn into(self) -> Pair<'a, Rule> {
    self.token
  }
}

pub struct TokensIter<'a> {
  tokens: Pairs<'a, Rule>,
  source_file: &'a str,
}

impl<'a> TokensIter<'a> {
  fn new(tokens: Pairs<'a, Rule>, source_file: &'a str) -> Self {
    TokensIter { tokens, source_file }
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
    self.tokens.next().map(|raw_token| Token::new(raw_token, self.source_file))
  }
}

impl<'a> DoubleEndedIterator for TokensIter<'a> {
  fn next_back(&mut self) -> Option<Self::Item> {
    self.tokens.next_back().map(|raw_token| Token::new(raw_token, self.source_file))
  }
}

impl<'a> Into<Pairs<'a, Rule>> for TokensIter<'a> {
  fn into(self) -> Pairs<'a, Rule> {
    self.tokens
  }
}
