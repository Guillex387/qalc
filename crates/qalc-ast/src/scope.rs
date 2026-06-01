use std::{cell::RefCell, collections::HashMap, rc::Rc};

use qalc_utils::Span;

use crate::typed_ast::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
  Type,
  Function,
  Value,
  Parameter,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
  pub name: String,
  pub kind: SymbolKind,
  pub declared_type: Type,
  pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Scope {
  parent: Option<Rc<Scope>>,
  symbols: RefCell<HashMap<String, Vec<SymbolInfo>>>,
}

impl Scope {
  pub fn new(parent: Option<Rc<Scope>>) -> Self {
    Scope {
      parent,
      symbols: RefCell::new(HashMap::new()),
    }
  }

  pub fn define(&self, symbol: SymbolInfo) -> bool {
    let mut symbols = self.symbols.borrow_mut();
    let entry = symbols.entry(symbol.name.clone()).or_insert_with(Vec::new);

    if entry
      .iter()
      .any(|s| s.declared_type == symbol.declared_type)
    {
      false
    } else {
      entry.push(symbol);
      true
    }
  }

  pub fn resolve(&self, name: &str) -> Option<SymbolInfo> {
    let local_result = self
      .symbols
      .borrow()
      .get(name)
      .and_then(|vec| vec.first().cloned());

    if local_result.is_some() {
      return local_result;
    }

    if let Some(parent) = &self.parent {
      parent.resolve(name)
    } else {
      None
    }
  }

  pub fn resolve_declaration_overloaded(
    &self,
    name: &str,
    declared_type: &Type,
  ) -> Option<SymbolInfo> {
    if let Some(vec) = self.symbols.borrow().get(name) {
      if let Some(found) = vec.iter().find(|s| &s.declared_type == declared_type) {
        return Some(found.clone());
      }
    }

    if let Some(parent) = &self.parent {
      parent.resolve_declaration_overloaded(name, declared_type)
    } else {
      None
    }
  }
}
