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
  symbols: RefCell<HashMap<String, SymbolInfo>>,
}

impl Scope {
  pub fn new(parent: Option<Rc<Scope>>) -> Self {
    Scope {
      parent,
      symbols: RefCell::new(HashMap::new()),
    }
  }

  pub fn define(&self, symbol: SymbolInfo) -> bool {
    self
      .symbols
      .borrow_mut()
      .insert(symbol.name.clone(), symbol)
      .is_none()
  }

  pub fn resolve(&self, name: String) -> Option<SymbolInfo> {
    let local_result = self.symbols.borrow().get(&name).cloned();

    if local_result.is_some() {
      return local_result;
    }

    if let Some(parent) = &self.parent {
      parent.resolve(name)
    } else {
      None
    }
  }
}
