use std::{cell::RefCell, collections::HashMap, rc::Rc};

use qalc_utils::{CompilerResult, Diagnostic, Span};

use crate::typed_ast::{Type, TypeWithUnknow};

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
  pub infer_type: TypeWithUnknow,
  pub span: Span,
}

impl SymbolInfo {
  pub fn ttype(&self) -> CompilerResult<Type> {
    match self.infer_type.clone().try_into() {
      Ok(ttype) => Ok(ttype),
      Err(_) => Err(Diagnostic::error(
        "Type inference failed for this definition.",
        "Type not inferred — add an explicit type annotation here.",
        self.span.clone(),
      )),
    }
  }
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
    if symbols.get(&symbol.name).is_some() {
      false
    } else {
      symbols.insert(symbol.name.clone(), vec![symbol]);

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

  pub fn define_func(&self, symbol: SymbolInfo) -> bool {
    todo!()
  }

  pub fn resolve_call(&self, name: &str, ttype: &Type) -> Option<SymbolInfo> {
    todo!()
  }
}
