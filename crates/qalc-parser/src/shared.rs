use std::rc::Rc;

use qalc_utils::{Diagnostic, Source, Span};

use crate::tokenizer::Rule;

pub fn pest_error_to_diagnostic(
  source: &Rc<Source>,
  error: pest::error::Error<Rule>,
) -> Diagnostic {
  let span = match error.location {
    pest::error::InputLocation::Span((start, end)) => Span::new(source, start, end),
    pest::error::InputLocation::Pos(position) => Span::new(source, position, position + 1),
  };
  let label = error.variant.message().to_string();

  Diagnostic::error("Parsing error", label, span)
}
