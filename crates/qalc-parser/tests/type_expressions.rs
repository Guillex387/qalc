use std::rc::Rc;

use qalc_ast::base_ast::{TypeExpressionKind};
use qalc_ast::concrete_ast::ConcreteDeclarationKind;
use qalc_ast::Spanned;
use qalc_parser::parse_program;
use qalc_utils::Source;

fn parse_type_annotation(decl_src: &str) -> TypeExpressionKind<Spanned> {
  let source = Rc::new(Source::memory(decl_src.to_string()));
  let program = parse_program(&source).expect("parse program");

  let declaration = program
    .declarations
    .into_iter()
    .next()
    .expect("one declaration");

  match declaration.kind {
    ConcreteDeclarationKind::Function {
      type_annotation, ..
    } => type_annotation.kind,
    _ => panic!("expected function declaration"),
  }
}

fn assert_type_identifier(kind: &TypeExpressionKind<Spanned>, expected: &str) {
  match kind {
    TypeExpressionKind::Identifier(id) => assert_eq!(id, expected, "type identifier"),
    other => panic!("expected type identifier, got {:?}", other),
  }
}

#[test]
fn parse_type_identifier() {
  let kind = parse_type_annotation("f :: num f = 1;");
  assert_type_identifier(&kind, "num");
}

#[test]
fn parse_type_with_params() {
  let kind = parse_type_annotation("f :: list<num> f = 1;");
  match kind {
    TypeExpressionKind::IdentifierWithParams { identifier, params } => {
      assert_eq!(identifier, "list");
      assert_eq!(params.len(), 1);
      assert_type_identifier(&params[0].kind, "num");
    }
    other => panic!("expected IdentifierWithParams, got {:?}", other),
  }
}

#[test]
fn parse_type_slice() {
  let kind = parse_type_annotation("f :: [num] f = 1;");
  match kind {
    TypeExpressionKind::Slice(inner) => assert_type_identifier(&inner.kind, "num"),
    other => panic!("expected slice type, got {:?}", other),
  }
}

#[test]
fn parse_type_tuple() {
  let kind = parse_type_annotation("f :: (num, bool) f = 1;");
  match kind {
    TypeExpressionKind::Tuple(types) => {
      assert_eq!(types.len(), 2);
      assert_type_identifier(&types[0].kind, "num");
      assert_type_identifier(&types[1].kind, "bool");
    }
    other => panic!("expected tuple type, got {:?}", other),
  }
}

#[test]
fn parse_type_arrow() {
  let kind = parse_type_annotation("f :: num -> bool f = 1;");
  match kind {
    TypeExpressionKind::Transform {
      input_type,
      output_type,
    } => {
      assert_type_identifier(&input_type.kind, "num");
      assert_type_identifier(&output_type.kind, "bool");
    }
    other => panic!("expected arrow type, got {:?}", other),
  }
}

#[test]
fn parse_type_arrow_right_associative() {
  let kind = parse_type_annotation("f :: num -> num -> bool f = 1;");
  match kind {
    TypeExpressionKind::Transform {
      input_type,
      output_type,
    } => {
      assert_type_identifier(&input_type.kind, "num");
      match output_type.kind {
        TypeExpressionKind::Transform {
          input_type: inner_input,
          output_type: inner_output,
        } => {
          assert_type_identifier(&inner_input.kind, "num");
          assert_type_identifier(&inner_output.kind, "bool");
        }
        other => panic!("expected inner arrow type, got {:?}", other),
      }
    }
    other => panic!("expected arrow type, got {:?}", other),
  }
}

#[test]
fn parse_type_with_two_params() {
  let kind = parse_type_annotation("f :: map<num bool> f = 1;");
  match kind {
    TypeExpressionKind::IdentifierWithParams { identifier, params } => {
      assert_eq!(identifier, "map");
      assert_eq!(params.len(), 2);
      assert_type_identifier(&params[0].kind, "num");
      assert_type_identifier(&params[1].kind, "bool");
    }
    other => panic!("expected IdentifierWithParams, got {:?}", other),
  }
}
