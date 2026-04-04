use std::rc::Rc;

use qalc_ast::concrete_ast::{DeclarationKind, Pattern};
use qalc_parser::parse_program;
use qalc_utils::Source;

fn parse_params(params: &str) -> Vec<Pattern> {
  let code = format!("f :: num f {} = 0", params);
  let source = Rc::new(Source::memory(code));
  let program = parse_program(&source).expect("parse program");

  let declaration = program
    .declarations
    .into_iter()
    .next()
    .expect("one declaration");

  match declaration.kind {
    DeclarationKind::Function { params, .. } => params,
    _ => panic!("expected function declaration"),
  }
}

fn assert_pattern_eq(actual: &Pattern, expected: &Pattern) {
  match (actual, expected) {
    (Pattern::Wildcard, Pattern::Wildcard) => {}
    (Pattern::Variable(actual_name), Pattern::Variable(expected_name)) => {
      assert_eq!(actual_name, expected_name, "variable pattern");
    }
    (Pattern::Tuple(actual_items), Pattern::Tuple(expected_items)) => {
      assert_eq!(
        actual_items.len(),
        expected_items.len(),
        "tuple pattern length"
      );

      for (actual_item, expected_item) in actual_items.iter().zip(expected_items.iter()) {
        assert_pattern_eq(actual_item, expected_item);
      }
    }
    _ => panic!("expected {:?}, got {:?}", expected, actual),
  }
}

#[test]
fn parse_variable_and_wildcard_patterns() {
  let params = parse_params("_ x");
  assert_eq!(params.len(), 2, "params length");

  assert_pattern_eq(&params[0], &Pattern::Wildcard);
  assert_pattern_eq(&params[1], &Pattern::Variable("x".to_string()));
}

#[test]
fn parse_tuple_pattern() {
  let params = parse_params("(a, b)");
  assert_eq!(params.len(), 1, "params length");

  let expected = Pattern::Tuple(vec![
    Pattern::Variable("a".to_string()),
    Pattern::Variable("b".to_string()),
  ]);

  assert_pattern_eq(&params[0], &expected);
}

#[test]
fn parse_nested_tuple_patterns() {
  let params = parse_params("((a, _), (b, (c, d)))");
  assert_eq!(params.len(), 1, "params length");

  let expected = Pattern::Tuple(vec![
    Pattern::Tuple(vec![Pattern::Variable("a".to_string()), Pattern::Wildcard]),
    Pattern::Tuple(vec![
      Pattern::Variable("b".to_string()),
      Pattern::Tuple(vec![
        Pattern::Variable("c".to_string()),
        Pattern::Variable("d".to_string()),
      ]),
    ]),
  ]);

  assert_pattern_eq(&params[0], &expected);
}
