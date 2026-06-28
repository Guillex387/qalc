use std::rc::Rc;

use qalc_ast::scope::{Scope, SymbolInfo, SymbolKind};
use qalc_ast::typed_ast::TypeWithUnknow;
use qalc_utils::{Source, Span};

fn dummy_span() -> Span {
  let source = Rc::new(Source::memory(String::new()));
  Span::new(&source, 0, 0)
}

fn make_symbol(name: &str) -> SymbolInfo {
  SymbolInfo {
    name: name.to_string(),
    kind: SymbolKind::Value,
    infer_type: TypeWithUnknow::Builtin {
      id: "i32".to_string(),
      params: vec![],
      linear: false,
    },
    span: dummy_span(),
  }
}

#[test]
fn scope_define_and_resolve() {
  let scope = Scope::new(None);
  let defined = scope.define(make_symbol("x"));
  assert!(defined, "first define must succeed");

  let resolved = scope.resolve("x");
  assert!(resolved.is_some(), "resolved must find the symbol");
  assert_eq!(resolved.unwrap().name, "x");
}

#[test]
fn scope_resolve_undefined_returns_none() {
  let scope = Scope::new(None);
  assert!(scope.resolve("undefined_var").is_none());
}

#[test]
fn scope_define_duplicate_fails() {
  let scope = Scope::new(None);
  assert!(scope.define(make_symbol("x")), "first define");
  assert!(!scope.define(make_symbol("x")), "duplicate define must return false");
}

#[test]
fn scope_child_resolves_parent() {
  let parent = Rc::new(Scope::new(None));
  parent.define(make_symbol("parentVar"));

  let child = Scope::new(Some(Rc::clone(&parent)));
  let resolved = child.resolve("parentVar");
  assert!(resolved.is_some(), "child must see parent symbols");
  assert_eq!(resolved.unwrap().name, "parentVar");
}

#[test]
fn scope_child_shadows_parent() {
  let parent = Rc::new(Scope::new(None));
  parent.define(SymbolInfo {
    name: "x".to_string(),
    kind: SymbolKind::Value,
    infer_type: TypeWithUnknow::Builtin {
      id: "i32".to_string(),
      params: vec![],
      linear: false,
    },
    span: dummy_span(),
  });

  let child = Scope::new(Some(Rc::clone(&parent)));
  child.define(SymbolInfo {
    name: "x".to_string(),
    kind: SymbolKind::Parameter,
    infer_type: TypeWithUnknow::Builtin {
      id: "f64".to_string(),
      params: vec![],
      linear: false,
    },
    span: dummy_span(),
  });

  let resolved = child.resolve("x").unwrap();
  assert_eq!(resolved.kind, SymbolKind::Parameter, "child symbol shadows parent");
}

#[test]
fn scope_define_decl_allows_overloading() {
  let scope = Scope::new(None);
  assert!(scope.define_decl(make_symbol("f")), "first define_decl");
  assert!(scope.define_decl(make_symbol("f")), "second define_decl with same name");

  let resolved = scope.resolve("f");
  assert!(resolved.is_some(), "must resolve the first overload");
}

#[test]
fn scope_parent_not_visible_to_sibling() {
  let parent = Rc::new(Scope::new(None));
  let child_a = Scope::new(Some(Rc::clone(&parent)));
  child_a.define(make_symbol("a_only"));

  let child_b = Scope::new(Some(Rc::clone(&parent)));
  assert!(
    child_b.resolve("a_only").is_none(),
    "sibling scope must not see symbols from another child"
  );
}
