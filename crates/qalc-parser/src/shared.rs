use pest::iterators::Pair;
use qalc_ast::concrete_ast;

use crate::Rule;

pub fn token_to_span(pair: &Pair<Rule>) -> concrete_ast::Span {
  let pest_span = pair.as_span();
  concrete_ast::Span::new(pest_span.start(), pest_span.end())
}
