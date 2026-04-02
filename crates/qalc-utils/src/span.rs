use std::{ fs, ops::Range };

use ariadne::Source;

#[derive(Debug, Clone)]
pub struct Span {
  pub filename: String,
  pub start: usize,
  pub end: usize,
}

impl Span {
  pub fn new(filename: impl Into<String>, start: usize, end: usize) -> Self {
    Span { filename: filename.into(), start, end }
  }

  pub fn to_ariadne_span(&self) -> (&str, Range<usize>) {
    (&self.filename, self.start..self.end)
  }

  pub fn to_ariadne_source(&self) -> (&str, Source) {
    let source_code = fs::read_to_string(&self.filename).unwrap();

    (&self.filename, Source::from(source_code))
  }
}
