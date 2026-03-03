use pest::iterators::Pair;
use qalc_ast::Span;

use crate::Rule;

pub fn token_to_span(pair: &Pair<Rule>) -> Span {
  let pest_span = pair.as_span();
  Span::new(pest_span.start(), pest_span.end())
}
