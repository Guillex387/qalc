use std::rc::Rc;

use qalc_ast::base_ast::PatternKind;
use qalc_ast::concrete_ast::{ConcreteDeclarationKind, ConcreteExpressionKind};
use qalc_parser::parse_program;
use qalc_utils::Source;

fn parse_function(src: &str) -> ConcreteDeclarationKind {
  let source = Rc::new(Source::memory(src.to_string()));
  let program = parse_program(&source).expect("parse program");

  program
    .declarations
    .into_iter()
    .next()
    .expect("one declaration")
    .kind
}

#[test]
fn parse_function_no_params() {
  let kind = parse_function("f :: num f = 1;");
  match kind {
    ConcreteDeclarationKind::Function {
      identifier,
      params,
      body,
      ..
    } => {
      assert_eq!(identifier, "f");
      assert!(params.is_empty(), "expected no params");
      match body.kind {
        ConcreteExpressionKind::IntegerLiteral { number, .. } => {
          assert_eq!(number, 1);
        }
        other => panic!("expected integer literal body, got {:?}", other),
      }
    }
    other => panic!("expected function declaration, got {:?}", other),
  }
}

#[test]
fn parse_function_single_param() {
  let kind = parse_function("f :: num -> num f x = x;");
  match kind {
    ConcreteDeclarationKind::Function {
      identifier,
      params,
      body,
      ..
    } => {
      assert_eq!(identifier, "f");
      assert_eq!(params.len(), 1);
      match &params[0].kind {
        PatternKind::Variable(name) => assert_eq!(name, "x"),
        other => panic!("expected variable pattern, got {:?}", other),
      }
      match body.kind {
        ConcreteExpressionKind::Identifier(name) => assert_eq!(name, "x"),
        other => panic!("expected identifier body, got {:?}", other),
      }
    }
    other => panic!("expected function declaration, got {:?}", other),
  }
}

#[test]
fn parse_function_multi_param() {
  let kind = parse_function("f :: num -> num -> num f x y = x;");
  match kind {
    ConcreteDeclarationKind::Function { params, .. } => {
      assert_eq!(params.len(), 2);
      match &params[0].kind {
        PatternKind::Variable(name) => assert_eq!(name, "x"),
        other => panic!("expected variable pattern for x, got {:?}", other),
      }
      match &params[1].kind {
        PatternKind::Variable(name) => assert_eq!(name, "y"),
        other => panic!("expected variable pattern for y, got {:?}", other),
      }
    }
    other => panic!("expected function declaration, got {:?}", other),
  }
}

#[test]
fn parse_function_tuple_param() {
  let kind = parse_function("f :: (num, num) -> num f (x, y) = x;");
  match kind {
    ConcreteDeclarationKind::Function { params, body, .. } => {
      assert_eq!(params.len(), 1);
      match &params[0].kind {
        PatternKind::Tuple(inner) => {
          assert_eq!(inner.len(), 2);
          match &inner[0].kind {
            PatternKind::Variable(name) => assert_eq!(name, "x"),
            other => panic!("expected variable pattern for x, got {:?}", other),
          }
          match &inner[1].kind {
            PatternKind::Variable(name) => assert_eq!(name, "y"),
            other => panic!("expected variable pattern for y, got {:?}", other),
          }
        }
        other => panic!("expected tuple pattern, got {:?}", other),
      }
      match body.kind {
        ConcreteExpressionKind::Identifier(name) => assert_eq!(name, "x"),
        other => panic!("expected identifier body, got {:?}", other),
      }
    }
    other => panic!("expected function declaration, got {:?}", other),
  }
}

#[test]
fn parse_function_wildcard_param() {
  let kind = parse_function("f :: num -> num f _ = 1;");
  match kind {
    ConcreteDeclarationKind::Function { params, .. } => {
      assert_eq!(params.len(), 1);
      match &params[0].kind {
        PatternKind::Wildcard => {}
        other => panic!("expected wildcard pattern, got {:?}", other),
      }
    }
    other => panic!("expected function declaration, got {:?}", other),
  }
}

#[test]
fn parse_function_identifier() {
  let kind = parse_function("myFunc :: num myFunc = 1;");
  match kind {
    ConcreteDeclarationKind::Function { identifier, .. } => {
      assert_eq!(identifier, "myFunc");
    }
    other => panic!("expected function declaration, got {:?}", other),
  }
}

#[test]
fn parse_multiple_declarations() {
  let source = Rc::new(Source::memory("f :: num f = 1; g :: num g = 2;".to_string()));
  let program = parse_program(&source).expect("parse program");
  assert_eq!(program.declarations.len(), 2);

  match &program.declarations[0].kind {
    ConcreteDeclarationKind::Function { identifier, .. } => assert_eq!(identifier, "f"),
    other => panic!("expected function for f, got {:?}", other),
  }
  match &program.declarations[1].kind {
    ConcreteDeclarationKind::Function { identifier, .. } => assert_eq!(identifier, "g"),
    other => panic!("expected function for g, got {:?}", other),
  }
}
