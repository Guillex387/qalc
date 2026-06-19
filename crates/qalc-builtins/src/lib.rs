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

fn ty(id: &str) -> TypeWithUnknow {
  TypeWithUnknow::Builtin {
    id: id.to_string(),
    params: vec![],
    linear: false,
  }
}

fn arrow(from: TypeWithUnknow, to: TypeWithUnknow) -> TypeWithUnknow {
  TypeWithUnknow::Arrow(Box::new(from), Box::new(to))
}

pub fn standard_builtins() -> Vec<BuiltinDef> {
  let i32 = || ty("i32");
  let f64 = || ty("f64");
  let bool = || ty("bool");

  vec![
    // i32 arithmetic
    BuiltinDef::new("add", arrow(i32(), arrow(i32(), i32()))),
    BuiltinDef::new("sub", arrow(i32(), arrow(i32(), i32()))),
    BuiltinDef::new("mul", arrow(i32(), arrow(i32(), i32()))),
    BuiltinDef::new("div", arrow(i32(), arrow(i32(), i32()))),
    BuiltinDef::new("neg", arrow(i32(), i32())),
    // i32 comparison
    BuiltinDef::new("eq_i32", arrow(i32(), arrow(i32(), bool()))),
    BuiltinDef::new("lt_i32", arrow(i32(), arrow(i32(), bool()))),
    // f64 arithmetic
    BuiltinDef::new("addf", arrow(f64(), arrow(f64(), f64()))),
    BuiltinDef::new("subf", arrow(f64(), arrow(f64(), f64()))),
    BuiltinDef::new("mulf", arrow(f64(), arrow(f64(), f64()))),
    BuiltinDef::new("divf", arrow(f64(), arrow(f64(), f64()))),
    BuiltinDef::new("negf", arrow(f64(), f64())),
  ]
}
