use std::{fs, ops::Range};

use ariadne::Source;

#[derive(Debug, Clone)]
pub struct Span {
  pub source_file: String,
  pub start: usize,
  pub end: usize,
}

impl Span {
  pub fn new(source_file: impl Into<String>, start: usize, end: usize) -> Self {
    Span {
      source_file: source_file.into(),
      start,
      end,
    }
  }

  pub fn join(&self, span: &Span) -> Span {
    Span {
      source_file: self.source_file.clone(),
      start: self.start,
      end: span.end,
    }
  }

  pub fn to_ariadne_span(&self) -> (&str, Range<usize>) {
    (&self.source_file, self.start..self.end)
  }

  pub fn to_ariadne_source(&self) -> (&str, Source) {
    let source_code = fs::read_to_string(&self.source_file).unwrap() + " ";

    (&self.source_file, Source::from(source_code))
  }
}
