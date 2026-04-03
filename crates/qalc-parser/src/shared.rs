use qalc_utils::{Diagnostic, Span};

use crate::Rule;

pub fn pest_error_to_diagnostic(filename: &str, error: pest::error::Error<Rule>) -> Diagnostic {
  let span = match error.location {
    pest::error::InputLocation::Span((start, end)) => Span::new(filename, start, end),
    pest::error::InputLocation::Pos(position) => Span::new(filename, position, position + 1),
  };
  let message = error.variant.message().to_string();

  Diagnostic::error("Parsing error", message, span)
}
