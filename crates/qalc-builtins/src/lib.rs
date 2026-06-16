use qalc_ast::typed_ast::TypeWithUnknow;

pub struct BuiltinDef {
  pub name: String,
  pub ttype: TypeWithUnknow,
}

impl BuiltinDef {
  pub fn new(name: impl Into<String>, ttype: TypeWithUnknow) -> Self {
    Self {
      name: name.into(),
      ttype,
    }
  }
}
