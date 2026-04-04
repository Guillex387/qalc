use std::{ops::Range, rc::Rc};

use crate::source::Source;

#[derive(Debug, Clone)]
pub struct Span {
  source: Rc<Source>,
  pub start: usize,
  pub end: usize,
}

impl Span {
  pub fn new(source: &Rc<Source>, start: usize, end: usize) -> Self {
    Span {
      source: source.clone(),
      start,
      end,
    }
  }

  pub fn source(&self) -> &Rc<Source> {
    &self.source
  }

  pub fn join(&self, span: &Span) -> Span {
    Span {
      source: self.source.clone(),
      start: self.start,
      end: span.end,
    }
  }

  pub fn to_ariadne_span(&self) -> (&str, Range<usize>) {
    (&self.source.name(), self.start..self.end)
  }
}
