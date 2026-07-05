use qalc_ast::typed_ast::{TypeWithUnknow, is_linear_builtin};

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
    linear: is_linear_builtin(id),
  }
}

/// Constructor de tipo built-in con parámetros (por ejemplo `qio<u8>`).
fn ty_params(id: &str, params: Vec<TypeWithUnknow>) -> TypeWithUnknow {
  TypeWithUnknow::Builtin {
    id: id.to_string(),
    params,
    linear: is_linear_builtin(id),
  }
}

fn arrow(from: TypeWithUnknow, to: TypeWithUnknow) -> TypeWithUnknow {
  TypeWithUnknow::Arrow(Box::new(from), Box::new(to))
}

/// Construye una firma currificada a partir de una lista de tipos de parámetro
/// y un tipo de retorno, por ejemplo `arrows([i32, qcirc], qcirc)` produce
/// `i32 -> qcirc -> qcirc`.
fn arrows(params: Vec<TypeWithUnknow>, ret: TypeWithUnknow) -> TypeWithUnknow {
  params.into_iter().rev().fold(ret, |acc, p| arrow(p, acc))
}

pub fn standard_builtins() -> Vec<BuiltinDef> {
  let i32 = || ty("i32");
  let i16 = || ty("i16");
  let f64 = || ty("f64");
  let bool = || ty("bool");
  let u8 = || ty("u8");
  let qcirc = || ty("qcirc");
  let qi16 = || ty("qi16");
  let qbool = || ty("qbool");
  let qu8 = || ty("qu8");
  // Una puerta es una transformación del circuito completo, `qcirc -> qcirc`.
  let gate = || arrow(qcirc(), qcirc());
  // El tipo de efecto de una medición, `qio<u8>`.
  let qio_u8 = || ty_params("qio", vec![u8()]);
  let unit = || TypeWithUnknow::Tuple(vec![]);
  let pair = |a, b| TypeWithUnknow::Tuple(vec![a, b]);
  let triple = |a, b, c| TypeWithUnknow::Tuple(vec![a, b, c]);

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
    // Puertas básicas de un solo cúbit sobre el circuito completo (qcirc -> qcirc)
    BuiltinDef::new("H", gate()),
    BuiltinDef::new("X", gate()),
    BuiltinDef::new("Y", gate()),
    BuiltinDef::new("Z", gate()),
    // Aplicación posicional de una puerta a un cúbit concreto
    BuiltinDef::new("at", arrows(vec![i32(), gate()], gate())),
    // Puerta controlada (CNOT) y doblemente controlada (Toffoli)
    BuiltinDef::new("controlled", arrows(vec![i32(), i32(), gate()], gate())),
    BuiltinDef::new("toffoli", arrows(vec![i32(), i32(), i32(), gate()], gate())),
    // Puertas de rotación parametrizadas por un ángulo
    BuiltinDef::new("rx", arrows(vec![f64(), qcirc()], qcirc())),
    BuiltinDef::new("ry", arrows(vec![f64(), qcirc()], qcirc())),
    BuiltinDef::new("rz", arrows(vec![f64(), qcirc()], qcirc())),
    // Medición, la operación de efecto que produce un resultado clásico
    // envuelto en el tipo de efecto `qio`. Hay una versión por tipo de dato
    // cuántico medible porque la sobrecarga de funciones todavía no está
    // implementada.
    BuiltinDef::new("measure", arrow(qcirc(), qio_u8())),
    BuiltinDef::new("measureBool", arrow(qbool(), qio_u8())),
    // Combinadores monádicos de efecto usados por la desugarización de `perform`.
    // El sistema de tipos actual es monomórfico, así que se instancian sobre
    // `u8`; una demostración de efectos con `perform` funciona para ese caso.
    BuiltinDef::new(
      "monadMap",
      arrows(vec![qio_u8(), arrow(u8(), qio_u8())], qio_u8()),
    ),
    BuiltinDef::new(
      "flatMap",
      arrows(vec![qio_u8(), arrow(u8(), qio_u8())], qio_u8()),
    ),
    BuiltinDef::new("monadPure", arrow(u8(), qio_u8())),
    BuiltinDef::new("pure", arrow(u8(), qio_u8())),
    // `uncompute` libera un recurso auxiliar; es una operación de solo efecto.
    // Hay una versión por tipo de dato cuántico porque la sobrecarga de
    // funciones todavía no está implementada.
    BuiltinDef::new("uncompute", arrow(qi16(), unit())),
    BuiltinDef::new("uncomputeQbool", arrow(qbool(), unit())),
    BuiltinDef::new("uncomputeQu8", arrow(qu8(), unit())),
    // Constructores de datos cuánticos: un built-in con el mismo nombre que el
    // tipo que construye, que recibe un argumento clásico y produce un valor
    // cuántico fresco. En el caso de `qcirc` ese argumento es el tamaño del
    // registro, de modo que su anchura queda determinada unívocamente en el
    // punto de construcción; en `qi16`, `qu8` y `qbool` es el valor clásico con
    // el que se inicializa el registro (`qbool` toma un `i32` en vez de un
    // `bool` porque el lenguaje todavía no tiene sintaxis de literal booleano;
    // 0 es falso y cualquier otro valor es verdadero). `qu8` reserva siempre un
    // registro de 2 cúbits. La generación de código exige que ese argumento sea
    // una constante conocida en tiempo de compilación.
    BuiltinDef::new("qcirc", arrow(i32(), qcirc())),
    BuiltinDef::new("qi16", arrow(i16(), qi16())),
    BuiltinDef::new("qbool", arrow(i32(), qbool())),
    BuiltinDef::new("qu8", arrow(u8(), qu8())),
    // Conversiones de identidad entre los tipos cuánticos de alto nivel
    // (`qu8`, `qbool`) y `qcirc`. Permiten aplicar las puertas de bajo nivel
    // (H, X, toffoli, at, ...) a un valor de alto nivel, y devolver el
    // resultado etiquetado de nuevo con su tipo de alto nivel. No emiten
    // ninguna instrucción: comparten la misma representación en tiempo de
    // ejecución.
    BuiltinDef::new("qcircOfQu8", arrow(qu8(), qcirc())),
    BuiltinDef::new("qu8OfQcirc", arrow(qcirc(), qu8())),
    BuiltinDef::new("qcircOfQbool", arrow(qbool(), qcirc())),
    BuiltinDef::new("qboolOfQcirc", arrow(qcirc(), qbool())),
    // Puertas que cruzan registros: `toffoli`/`controlled` solo admiten un
    // único `qcirc` de entrada con índices locales a él, así que no pueden
    // expresar una puerta cuyo control y objetivo viven en registros
    // distintos (necesario para un sumador de varios operandos). Cada
    // operando se pasa como su propio par (registro, índice); se devuelven
    // los registros correspondientes rethreadados.
    BuiltinDef::new(
      "crossControlled",
      arrows(vec![qcirc(), i32(), qcirc(), i32()], pair(qcirc(), qcirc())),
    ),
    BuiltinDef::new(
      "crossToffoli",
      arrows(
        vec![qcirc(), i32(), qcirc(), i32(), qcirc(), i32()],
        triple(qcirc(), qcirc(), qcirc()),
      ),
    ),
    // Despliega `n` veces (constante conocida en tiempo de compilación) la
    // función de un cúbit dada, encadenando su salida como entrada de la
    // siguiente iteración. Es el único mecanismo de repetición del lenguaje,
    // ya que Qalc no admite recursión.
    BuiltinDef::new("repeat", arrows(vec![i32(), gate()], gate())),
  ]
}
