use std::rc::Rc;

use qalc_ast::Spanned;
use qalc_ast::base_ast::{Pattern, PatternKind};
use qalc_ast::concrete_ast::{ConcreteDeclarationKind, ConcreteExpressionKind};
use qalc_parser::parse_program;
use qalc_utils::Source;

fn parse_expression_kind(expression: &str) -> ConcreteExpressionKind {
  let code = format!("f :: num f = {};", expression);
  let source = Rc::new(Source::memory(code));
  let program = parse_program(&source).expect("parse program");

  let declaration = program
    .declarations
    .into_iter()
    .next()
    .expect("one declaration");

  match declaration.kind {
    ConcreteDeclarationKind::Function { body, .. } => body.kind,
    _ => panic!("expected function declaration"),
  }
}

fn assert_identifier(kind: &ConcreteExpressionKind, expected: &str) {
  match kind {
    ConcreteExpressionKind::Identifier(value) => {
      assert_eq!(value, expected, "identifier value");
    }
    other => panic!("expected identifier, got {:?}", other),
  }
}

fn assert_param_vars(params: &[Pattern<Spanned>], expected: &[&str]) {
  assert_eq!(params.len(), expected.len(), "params length");

  for (param, expected_name) in params.iter().zip(expected.iter()) {
    match &param.kind {
      PatternKind::Variable(name) => {
        assert_eq!(name, expected_name, "param name");
      }
      other => panic!("expected variable pattern, got {:?}", other),
    }
  }
}

#[derive(Debug)]
enum PatternShape {
  Variable(String),
  Tuple(Vec<PatternShape>),
  Wildcard,
}

fn assert_pattern_eq(actual: &Pattern<Spanned>, expected: &PatternShape) {
  match (&actual.kind, expected) {
    (PatternKind::Wildcard, PatternShape::Wildcard) => {}
    (PatternKind::Variable(actual_name), PatternShape::Variable(expected_name)) => {
      assert_eq!(actual_name, expected_name, "pattern variable");
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
fn parse_lambda_expression() {
  let kind = parse_expression_kind("(x y) -> x");

  match kind {
    ConcreteExpressionKind::Lambda { params, body } => {
      assert_param_vars(&params, &["x", "y"]);
      assert_identifier(&body.kind, "x");
    }
    other => panic!("expected lambda expression, got {:?}", other),
  }
}

#[test]
fn parse_lambda_expression_with_wildcard_param() {
  let kind = parse_expression_kind("(_ x) -> x");

  match kind {
    ConcreteExpressionKind::Lambda { params, body } => {
      assert_eq!(params.len(), 2, "params length");
      assert_pattern_eq(&params[0], &PatternShape::Wildcard);
      assert_pattern_eq(&params[1], &PatternShape::Variable("x".to_string()));
      assert_identifier(&body.kind, "x");
    }
    other => panic!("expected lambda expression, got {:?}", other),
  }
}

#[test]
fn parse_lambda_expression_with_tuple_param() {
  let kind = parse_expression_kind("((a, _)) -> a");

  let expected = PatternShape::Tuple(vec![
    PatternShape::Variable("a".to_string()),
    PatternShape::Wildcard,
  ]);

  match kind {
    ConcreteExpressionKind::Lambda { params, body } => {
      assert_eq!(params.len(), 1, "params length");
      assert_pattern_eq(&params[0], &expected);
      assert_identifier(&body.kind, "a");
    }
    other => panic!("expected lambda expression, got {:?}", other),
  }
}
