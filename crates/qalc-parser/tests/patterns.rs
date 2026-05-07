use std::rc::Rc;

use qalc_ast::Spanned;
use qalc_ast::base_ast::{Pattern, PatternKind};
use qalc_ast::concrete_ast::ConcreteDeclarationKind;
use qalc_parser::parse_program;
use qalc_utils::Source;

fn parse_params(params: &str) -> Vec<Pattern<Spanned>> {
  let code = format!("f :: num f {} = 0;", params);
  let source = Rc::new(Source::memory(code));
  let program = parse_program(&source).expect("parse program");

  let declaration = program
    .declarations
    .into_iter()
    .next()
    .expect("one declaration");

  match declaration.kind {
    ConcreteDeclarationKind::Function { params, .. } => params,
    _ => panic!("expected function declaration"),
  }
}

#[derive(Debug, Clone)]
enum PatternShape {
  Variable(String),
  Tuple(Vec<PatternShape>),
  Wildcard,
}

fn assert_pattern_eq(actual: &Pattern<Spanned>, expected: &PatternShape) {
  match (&actual.kind, expected) {
    (PatternKind::Wildcard, PatternShape::Wildcard) => {}
    (PatternKind::Variable(actual_name), PatternShape::Variable(expected_name)) => {
      assert_eq!(actual_name, expected_name, "variable pattern");
    }
    (PatternKind::Tuple(actual_items), PatternShape::Tuple(expected_items)) => {
      assert_eq!(
        actual_items.len(),
        expected_items.len(),
        "tuple pattern length"
      );

      for (actual_item, expected_item) in actual_items.iter().zip(expected_items.iter()) {
        assert_pattern_eq(actual_item, expected_item);
      }
    }
    _ => panic!("expected {:?}, got {:?}", expected, actual.kind),
  }
}

#[test]
fn parse_variable_and_wildcard_patterns() {
  let params = parse_params("_ x");
  assert_eq!(params.len(), 2, "params length");

  assert_pattern_eq(&params[0], &PatternShape::Wildcard);
  assert_pattern_eq(&params[1], &PatternShape::Variable("x".to_string()));
}

#[test]
fn parse_tuple_pattern() {
  let params = parse_params("(a, b)");
  assert_eq!(params.len(), 1, "params length");

  let expected = PatternShape::Tuple(vec![
    PatternShape::Variable("a".to_string()),
    PatternShape::Variable("b".to_string()),
  ]);

  assert_pattern_eq(&params[0], &expected);
}

#[test]
fn parse_nested_tuple_patterns() {
  let params = parse_params("((a, _), (b, (c, d)))");
  assert_eq!(params.len(), 1, "params length");

  let expected = PatternShape::Tuple(vec![
    PatternShape::Tuple(vec![
      PatternShape::Variable("a".to_string()),
      PatternShape::Wildcard,
    ]),
    PatternShape::Tuple(vec![
      PatternShape::Variable("b".to_string()),
      PatternShape::Tuple(vec![
        PatternShape::Variable("c".to_string()),
        PatternShape::Variable("d".to_string()),
      ]),
    ]),
  ]);

  assert_pattern_eq(&params[0], &expected);
}
