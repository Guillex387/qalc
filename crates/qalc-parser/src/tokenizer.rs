use std::rc::Rc;

use pest::{
  Parser,
  iterators::{Pair, Pairs},
};
use pest_derive::Parser;
use qalc_utils::{CompilerResult, Source, Span};

use crate::shared::pest_error_to_diagnostic;

#[derive(Debug, Clone)]
pub struct Token<'a> {
  token: Pair<'a, Rule>,
  source: Rc<Source>,
}

impl<'a> Token<'a> {
  pub fn new(token: Pair<'a, Rule>, source: &Rc<Source>) -> Token<'a> {
    Token {
      token,
      source: source.clone(),
    }
  }

  pub fn source(&self) -> &Rc<Source> {
    &self.source
  }

  pub fn rule(&self) -> Rule {
    self.token.as_rule()
  }

  pub fn span(&self) -> Span {
    let pest_span = self.token.as_span();
    Span::new(&self.source, pest_span.start(), pest_span.end())
  }

  pub fn inner(self) -> TokensIter<'a> {
    TokensIter::new(self.token.into_inner(), &self.source)
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
  source: Rc<Source>,
}

impl<'a> TokensIter<'a> {
  fn new(tokens: Pairs<'a, Rule>, source: &Rc<Source>) -> Self {
    TokensIter {
      tokens,
      source: source.clone(),
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
      .map(|raw_token| Token::new(raw_token, &self.source))
  }
}

impl DoubleEndedIterator for TokensIter<'_> {
  fn next_back(&mut self) -> Option<Self::Item> {
    self
      .tokens
      .next_back()
      .map(|raw_token| Token::new(raw_token, &self.source))
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

pub fn tokenize(source: &Rc<Source>) -> CompilerResult<TokensIter<'_>> {
  let mut program_tokens = TokensIter::new(
    QalcGrammar::parse(Rule::program, source.content())
      .map_err(|error| pest_error_to_diagnostic(source, error))?,
    source,
  );
  let mut declarations_tokens = program_tokens.eat().inner();
  declarations_tokens.eat_back(); // CONSUME EOF token

  Ok(declarations_tokens)
}
