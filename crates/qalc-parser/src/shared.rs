use pest::iterators::Pair;
use qalc_utils::{ Diagnostic, Span };

use crate::Rule;

pub fn token_to_span(filename: impl Into<String>, token: &Pair<Rule>) -> Span {
  let pest_span = token.as_span();
  Span::new(filename, pest_span.start(), pest_span.end())
}

pub fn token_to_string(token: Pair<Rule>) -> String {
  token.as_str().to_owned()
}

pub fn pest_error_to_diagnostic(
  filename: &str,
  error: pest::error::Error<Rule>
) -> Diagnostic {
  let span = match error.location {
    pest::error::InputLocation::Span((start, end)) => Span::new(filename, start, end),
    pest::error::InputLocation::Pos(position) =>
      dbg!(Span::new(filename, position, position + 1)),
  };
  let message = dbg!(error.variant.message().to_string());

  Diagnostic::error("Parsing error", message, span)
}
