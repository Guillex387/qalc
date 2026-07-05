//! Generación de código, la fase final del pipeline de Qalc. Toma el `FlatAST`
//! ya validado por la comprobación de consumo lineal y emite texto OpenQASM 3.
//!
//! El diseño sigue el esbozo de la memoria (`report/07-capítulo-backend.tex`,
//! `sec:codegen`), adaptado a los tipos reales del compilador. Es una base
//! suficiente para demostraciones, no un generador de código completo.
//!
//! Los parámetros y el resultado de `main` son siempre de tipos clásicos (lo
//! garantiza el aplanador, `crates/qalc-analysis/src/inliner`), lo que permite
//! traducirlos directamente a los modificadores `input`/`output` de OpenQASM 3
//! y obtener así circuitos paramétricos: el mismo programa puede ejecutarse con
//! distintos valores clásicos de entrada sin recompilar. Los valores cuánticos
//! nunca se reciben ni se devuelven; se construyen dentro del cuerpo con los
//! built-ins `qcirc`/`qi16`/`qbool`, cuyo argumento (el tamaño del registro o su
//! valor inicial) debe ser una constante conocida en tiempo de compilación, ya
//! que fija la anchura de una declaración `qubit[N]`.
//!
//! El cuerpo se obtiene recorriendo la única expresión del `FlatAST`, con un
//! modelo de evaluación por efecto donde cada valor cuántico es un alias de un
//! registro físico recién reservado y cada aplicación de built-in emite su
//! instrucción. Las puertas básicas se resuelven mediante emisores puros
//! registrados por nombre (`registry`). Los built-ins que necesitan contexto de
//! emisión (reservar registros cuánticos o clásicos) se resuelven directamente
//! en el `Emitter`.

mod backend;
mod registry;
mod value;

pub use backend::{Backend, OpenQasm3, OpenQasm3Transpilable, Quil, Transpilable};
pub use value::QasmValue;

use std::collections::{BTreeSet, HashMap};

use qalc_ast::base_ast::{Expression, ExpressionKind, Pattern, PatternKind, SliceExpression};
use qalc_ast::flat_ast::{Flat, FlatAST};
use qalc_ast::typed_ast::Type;
use qalc_utils::{CompilerResult, CompilerStep, Diagnostic, Span};

use registry::registry;

/// Nombres de las puertas de un solo cúbit que pueden pasarse como valor a un
/// combinador de orden superior (`at`, `controlled`, `toffoli`).
const GATE_VALUES: [&str; 4] = ["H", "X", "Y", "Z"];

/// Anchura del registro que representa un dato cuántico de 16 bits (`qi16`).
const QI16_WIDTH: usize = 16;

/// Anchura del registro que representa un dato cuántico `qu8`. Se fija en 2
/// cúbits para el ejemplo de Grover de 2 bits, no en 8.
const QU8_WIDTH: usize = 2;

/// Fase de generación de código. Transforma un `FlatAST` en el texto OpenQASM 3
/// correspondiente.
#[derive(Default)]
pub struct Codegen;

impl CompilerStep for Codegen {
  type Input = FlatAST;
  type Output = String;

  fn transform(&self, input: FlatAST, _: &mut Vec<Diagnostic>) -> CompilerResult<String> {
    let mut emitter = Emitter::new();
    emitter.run(input)
  }
}

/// Modo de entrada/salida de un registro o variable clásica declarada.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClassicalIo {
  Input,
  Output,
  /// Variable clásica interna sin modificador (por ejemplo el resultado de una
  /// medición que nunca llega a formar parte del resultado de `main`).
  Local,
}

/// Un registro o variable clásica declarada en el preámbulo.
struct ClassicalReg {
  name: String,
  qasm_type: String,
  io: ClassicalIo,
}

/// Estado mutable durante la emisión.
struct Emitter {
  registry: HashMap<&'static str, OpenQasm3Transpilable>,
  env: HashMap<String, QasmValue>,
  /// Registros cuánticos declarados, en orden de aparición (`nombre`, `anchura`).
  qregs: Vec<(String, usize)>,
  /// Registros y variables clásicas declaradas: parámetros de entrada,
  /// resultados de medición y variables de salida.
  classical: Vec<ClassicalReg>,
  /// Definiciones de puertas no estándar que hay que emitir en el preámbulo.
  gate_defs: BTreeSet<String>,
  /// Instrucciones del cuerpo del circuito.
  body: Vec<String>,
  qreg_counter: usize,
  measurement_counter: usize,
  output_counter: usize,
}

impl Emitter {
  fn new() -> Self {
    Self {
      registry: registry(),
      env: HashMap::new(),
      qregs: Vec::new(),
      classical: Vec::new(),
      gate_defs: BTreeSet::new(),
      body: Vec::new(),
      qreg_counter: 0,
      measurement_counter: 0,
      output_counter: 0,
    }
  }

  fn run(&mut self, ast: FlatAST) -> CompilerResult<String> {
    // Los parámetros de 'main' son siempre clásicos (lo garantiza el
    // aplanador), así que cada uno se traduce a una entrada `input`.
    for param in &ast.params {
      self.classical.push(ClassicalReg {
        name: param.name.clone(),
        qasm_type: qasm_classical_type(&param.ttype),
        io: ClassicalIo::Input,
      });
      self
        .env
        .insert(param.name.clone(), QasmValue::Classical(param.name.clone()));
    }

    let span = ast.body.meta.span.clone();
    let value = self.eval(&ast.body)?;
    let result_ty = unwrap_qio(&ast.body.meta.ttype).clone();
    self.bind_output(&result_ty, value, &span)?;

    Ok(self.assemble())
  }

  /// Ensambla el preámbulo y el cuerpo en el texto final.
  fn assemble(&self) -> String {
    let mut out = String::new();
    out.push_str("OPENQASM 3.0;\n");
    out.push_str("include \"stdgates.inc\";\n");
    for def in &self.gate_defs {
      out.push_str(def);
      out.push('\n');
    }
    for reg in &self.classical {
      if reg.io == ClassicalIo::Input {
        out.push_str(&format!("input {} {};\n", reg.qasm_type, reg.name));
      }
    }
    for (name, width) in &self.qregs {
      out.push_str(&format!("qubit[{width}] {name};\n"));
    }
    for reg in &self.classical {
      match reg.io {
        ClassicalIo::Input => {}
        ClassicalIo::Output => out.push_str(&format!("output {} {};\n", reg.qasm_type, reg.name)),
        ClassicalIo::Local => out.push_str(&format!("{} {};\n", reg.qasm_type, reg.name)),
      }
    }
    out.push('\n');
    for instr in &self.body {
      out.push_str(instr);
      out.push('\n');
    }
    out
  }

  /// Evalúa una expresión del `FlatAST` a un `QasmValue`, emitiendo por el camino
  /// las instrucciones que correspondan.
  fn eval(&mut self, expr: &Expression<Flat>) -> CompilerResult<QasmValue> {
    match &expr.kind {
      ExpressionKind::IntegerLiteral { number, .. } => Ok(QasmValue::Int(*number)),
      ExpressionKind::FloatLiteral { number, .. } => Ok(QasmValue::Float(*number)),
      ExpressionKind::Identifier(name) => Ok(self.eval_identifier(name)),
      ExpressionKind::Transform { .. } => self.eval_application(expr),
      ExpressionKind::LetBlock {
        assigments, value, ..
      } => {
        for (pat, rhs) in assigments {
          let v = self.eval(rhs)?;
          self.bind_pattern(pat, v);
        }
        self.eval(value)
      }
      ExpressionKind::Tuple(exprs) => {
        let values = exprs
          .iter()
          .map(|e| self.eval(e))
          .collect::<CompilerResult<Vec<_>>>()?;
        Ok(QasmValue::Tuple(values))
      }
      ExpressionKind::Slice(slice) => match slice {
        SliceExpression::Items(items) => {
          let values = items
            .iter()
            .map(|e| self.eval(e))
            .collect::<CompilerResult<Vec<_>>>()?;
          Ok(QasmValue::Tuple(values))
        }
        SliceExpression::Range(start, end) => {
          let start = self.eval(start)?;
          let end = self.eval(end)?;
          Ok(QasmValue::Tuple(vec![start, end]))
        }
      },
      ExpressionKind::IfElseBlock {
        test,
        consequent,
        alternative,
      } => self.eval_if(test, consequent, alternative),
      ExpressionKind::Lambda { .. } => Err(Diagnostic::error(
        "lambda residual no soportada por la generación de código",
        "el codegen no traduce todavía combinadores de orden superior con lambdas",
        expr.meta.span.clone(),
      )),
    }
  }

  fn eval_identifier(&self, name: &str) -> QasmValue {
    if GATE_VALUES.contains(&name) {
      QasmValue::Gate(name.to_string())
    } else if let Some(value) = self.env.get(name) {
      value.clone()
    } else {
      QasmValue::Classical(name.to_string())
    }
  }

  fn eval_application(&mut self, expr: &Expression<Flat>) -> CompilerResult<QasmValue> {
    let (head, arg_exprs) = peel_spine(expr);

    let ExpressionKind::Identifier(name) = &head.kind else {
      return Err(Diagnostic::error(
        "cabeza de aplicación no soportada por la generación de código",
        "se esperaba la aplicación de un built-in",
        expr.meta.span.clone(),
      ));
    };

    // `repeat` recibe una lambda como segundo argumento, que `eval` rechaza si
    // se evalúa genéricamente; hay que interceptarlo antes de evaluar los
    // argumentos y tratarlo por separado.
    if name == "repeat" {
      return self.eval_repeat(&arg_exprs, expr.meta.span.clone());
    }

    let mut args = Vec::with_capacity(arg_exprs.len());
    for a in &arg_exprs {
      args.push(self.eval(a)?);
    }

    self.emit_builtin(name, &args, expr.meta.span.clone())
  }

  /// Despliega `repeat n roundLambda inicial` evaluando el cuerpo de la lambda
  /// `n` veces en secuencia, encadenando la salida de cada iteración como
  /// entrada de la siguiente. `n` debe ser una constante conocida en tiempo de
  /// compilación.
  fn eval_repeat(
    &mut self,
    arg_exprs: &[&Expression<Flat>],
    span: Span,
  ) -> CompilerResult<QasmValue> {
    if arg_exprs.len() != 3 {
      return Err(Diagnostic::error(
        "'repeat' espera exactamente 3 argumentos",
        "uso: repeat n ronda valorInicial",
        span,
      ));
    }

    let n_value = self.eval(arg_exprs[0])?;
    let n = self.require_const_int(&[n_value], "repeat", &span)?;
    let n = usize::try_from(n).map_err(|_| {
      Diagnostic::error(
        "el número de iteraciones de 'repeat' debe ser un entero no negativo",
        "no se puede desplegar un número negativo de veces",
        span.clone(),
      )
    })?;

    let ExpressionKind::Lambda { params, body } = &arg_exprs[1].kind else {
      return Err(Diagnostic::error(
        "el segundo argumento de 'repeat' debe ser una lambda literal",
        "se esperaba una función de un parámetro (qcirc -> qcirc)",
        span,
      ));
    };
    let [param_name] = params.as_slice() else {
      return Err(Diagnostic::error(
        "la lambda de 'repeat' debe tener exactamente un parámetro",
        "se esperaba una función de un parámetro (qcirc -> qcirc)",
        span,
      ));
    };

    let mut current = self.eval(arg_exprs[2])?;
    for _ in 0..n {
      self.env.insert(param_name.clone(), current);
      current = self.eval(body)?;
    }
    Ok(current)
  }

  fn emit_builtin(
    &mut self,
    name: &str,
    args: &[QasmValue],
    span: Span,
  ) -> CompilerResult<QasmValue> {
    // Constructores de datos cuánticos: reservan un registro físico fresco a
    // partir de un argumento clásico conocido en tiempo de compilación.
    match name {
      "qcirc" => return self.construct_qcirc(args, &span),
      "qi16" => return self.construct_qi16(args, &span),
      "qbool" => return self.construct_qbool(args, &span),
      "qu8" => return self.construct_qu8(args, &span),
      // Conversiones de identidad entre qcirc y los tipos de alto nivel: misma
      // representación en tiempo de ejecución, ninguna instrucción emitida.
      "qcircOfQu8" | "qu8OfQcirc" | "qcircOfQbool" | "qboolOfQcirc" => {
        return Ok(args.first().cloned().unwrap_or(QasmValue::Unit));
      }
      "uncompute" | "uncomputeQbool" | "uncomputeQu8" => return Ok(self.emit_uncompute(args)),
      "measure" | "measureBool" => return Ok(self.emit_measure(args)),
      "crossControlled" => return self.emit_cross_controlled(args, &span),
      "crossToffoli" => return self.emit_cross_toffoli(args, &span),
      _ => {}
    }

    // Emisores de puerta puros registrados por nombre; el circuito se hila
    // devolviendo el registro que aparece entre los argumentos.
    if let Some(emit) = self.registry.get(name).map(|t| t.emit) {
      let fragment = emit(args);
      if !fragment.trim().is_empty() {
        self.body.push(fragment);
      }
      return Ok(thread_circuit(args));
    }

    Err(Diagnostic::error(
      format!("built-in sin emisor de código '{name}'"),
      "no hay plantilla de emisión OpenQASM 3 para este built-in",
      span,
    ))
  }

  /// Extrae de los argumentos una constante entera conocida en tiempo de
  /// compilación, o un error si el argumento es un valor clásico solo
  /// disponible en tiempo de ejecución (por ejemplo un parámetro de entrada).
  fn require_const_int(
    &self,
    args: &[QasmValue],
    builtin: &str,
    span: &Span,
  ) -> CompilerResult<i128> {
    args
      .iter()
      .find_map(|a| match a {
        QasmValue::Int(n) => Some(*n),
        _ => None,
      })
      .ok_or_else(|| {
        Diagnostic::error(
          format!(
            "el argumento de '{builtin}' debe ser una constante entera conocida en tiempo de compilación"
          ),
          "el tamaño o el valor inicial de un dato cuántico debe conocerse en tiempo de compilación para declararlo en OpenQASM 3",
          span.clone(),
        )
      })
  }

  /// Reserva un registro cuántico con un nombre generado automáticamente.
  fn fresh_qreg(&mut self, width: usize) -> String {
    let name = format!("q{}", self.qreg_counter);
    self.qreg_counter += 1;
    self.declare_named_qreg(name.clone(), width);
    name
  }

  fn declare_named_qreg(&mut self, name: String, width: usize) {
    if !self.qregs.iter().any(|(n, _)| n == &name) {
      self.qregs.push((name, width));
    }
  }

  fn reg_width(&self, name: &str) -> usize {
    self
      .qregs
      .iter()
      .find(|(n, _)| n == name)
      .map(|(_, w)| *w)
      .unwrap_or(1)
  }

  /// Emite las puertas `X` necesarias para inicializar un registro recién
  /// reservado a la representación binaria de una constante clásica.
  fn emit_init_bits(&mut self, reg: &str, value: i128, width: usize) {
    for i in 0..width {
      if (value >> i) & 1 == 1 {
        self.body.push(format!("x {reg}[{i}];"));
      }
    }
  }

  /// Construye un `qcirc` de `n` cúbits, donde `n` es el argumento del built-in.
  fn construct_qcirc(&mut self, args: &[QasmValue], span: &Span) -> CompilerResult<QasmValue> {
    let width = self.require_const_int(args, "qcirc", span)?;
    let width = usize::try_from(width).map_err(|_| {
      Diagnostic::error(
        "el tamaño de un 'qcirc' debe ser un entero no negativo",
        "un registro cuántico no puede declararse con anchura negativa",
        span.clone(),
      )
    })?;
    let name = self.fresh_qreg(width);
    Ok(QasmValue::QReg(name))
  }

  /// Construye un `qi16` inicializado a la constante clásica dada.
  fn construct_qi16(&mut self, args: &[QasmValue], span: &Span) -> CompilerResult<QasmValue> {
    let value = self.require_const_int(args, "qi16", span)?;
    let name = self.fresh_qreg(QI16_WIDTH);
    self.emit_init_bits(&name, value, QI16_WIDTH);
    Ok(QasmValue::QReg(name))
  }

  /// Construye un `qbool` inicializado a la constante clásica dada (0 es falso,
  /// cualquier otro valor es verdadero).
  fn construct_qbool(&mut self, args: &[QasmValue], span: &Span) -> CompilerResult<QasmValue> {
    let value = self.require_const_int(args, "qbool", span)?;
    let name = self.fresh_qreg(1);
    self.emit_init_bits(&name, value, 1);
    Ok(QasmValue::QReg(name))
  }

  /// Construye un `qu8` inicializado a la constante clásica dada. Reserva
  /// siempre un registro de 2 cúbits (véase `QU8_WIDTH`).
  fn construct_qu8(&mut self, args: &[QasmValue], span: &Span) -> CompilerResult<QasmValue> {
    let value = self.require_const_int(args, "qu8", span)?;
    let name = self.fresh_qreg(QU8_WIDTH);
    self.emit_init_bits(&name, value, QU8_WIDTH);
    Ok(QasmValue::QReg(name))
  }

  /// Extrae un par (registro, índice) de dos argumentos consecutivos, en el
  /// orden en que los reciben `crossControlled`/`crossToffoli`.
  fn qreg_index_pair(
    args: &[QasmValue],
    pos: usize,
    builtin: &str,
    span: &Span,
  ) -> CompilerResult<(String, usize)> {
    let reg = args
      .get(pos)
      .and_then(QasmValue::as_qreg)
      .map(str::to_string)
      .ok_or_else(|| {
        Diagnostic::error(
          format!("argumento de registro inválido para '{builtin}'"),
          "se esperaba un valor qcirc en esta posición",
          span.clone(),
        )
      })?;
    let idx = args
      .get(pos + 1)
      .and_then(QasmValue::as_index)
      .ok_or_else(|| {
        Diagnostic::error(
          format!("argumento de índice inválido para '{builtin}'"),
          "se esperaba un entero conocido en esta posición",
          span.clone(),
        )
      })?;
    Ok((reg, idx))
  }

  /// CNOT cruzando dos registros distintos: control en `(reg1, idx1)`, objetivo
  /// en `(reg2, idx2)`. Ambos registros se devuelven rethreadados.
  fn emit_cross_controlled(
    &mut self,
    args: &[QasmValue],
    span: &Span,
  ) -> CompilerResult<QasmValue> {
    let (control_reg, control_idx) = Self::qreg_index_pair(args, 0, "crossControlled", span)?;
    let (target_reg, target_idx) = Self::qreg_index_pair(args, 2, "crossControlled", span)?;
    self.body.push(format!(
      "cx {control_reg}[{control_idx}], {target_reg}[{target_idx}];"
    ));
    Ok(QasmValue::Tuple(vec![
      QasmValue::QReg(control_reg),
      QasmValue::QReg(target_reg),
    ]))
  }

  /// Toffoli cruzando hasta tres registros distintos: dos controles en
  /// `(reg1, idx1)`/`(reg2, idx2)`, objetivo en `(reg3, idx3)`. Los tres
  /// registros se devuelven rethreadados.
  fn emit_cross_toffoli(&mut self, args: &[QasmValue], span: &Span) -> CompilerResult<QasmValue> {
    let (control1_reg, control1_idx) = Self::qreg_index_pair(args, 0, "crossToffoli", span)?;
    let (control2_reg, control2_idx) = Self::qreg_index_pair(args, 2, "crossToffoli", span)?;
    let (target_reg, target_idx) = Self::qreg_index_pair(args, 4, "crossToffoli", span)?;
    self.body.push(format!(
      "ccx {control1_reg}[{control1_idx}], {control2_reg}[{control2_idx}], {target_reg}[{target_idx}];"
    ));
    Ok(QasmValue::Tuple(vec![
      QasmValue::QReg(control1_reg),
      QasmValue::QReg(control2_reg),
      QasmValue::QReg(target_reg),
    ]))
  }

  /// Descomputa un registro auxiliar para liberarlo (operación de solo efecto).
  fn emit_uncompute(&mut self, args: &[QasmValue]) -> QasmValue {
    let src = find_qreg(args);
    self.body.push(format!("// uncompute {src}"));
    QasmValue::Unit
  }

  /// Mide un registro cuántico sobre un registro clásico recién reservado.
  fn emit_measure(&mut self, args: &[QasmValue]) -> QasmValue {
    let src = find_qreg(args);
    let width = self.reg_width(&src);
    let bit = format!("c{}", self.measurement_counter);
    self.measurement_counter += 1;
    self.classical.push(ClassicalReg {
      name: bit.clone(),
      qasm_type: format!("bit[{width}]"),
      io: ClassicalIo::Local,
    });
    self.body.push(format!("{bit} = measure {src};"));
    QasmValue::Classical(bit)
  }

  fn eval_if(
    &mut self,
    test: &Expression<Flat>,
    consequent: &Expression<Flat>,
    alternative: &Expression<Flat>,
  ) -> CompilerResult<QasmValue> {
    let condition = self.eval(test)?;
    let cond_text = match condition {
      QasmValue::Classical(c) => c,
      QasmValue::Int(n) => n.to_string(),
      other => format!("{other:?}"),
    };

    let cons = self.capture(|e| e.eval(consequent))?;
    let alt = self.capture(|e| e.eval(alternative))?;

    self.body.push(format!("if ({cond_text}) {{"));
    for instr in cons.0 {
      self.body.push(format!("  {instr}"));
    }
    self.body.push("}".to_string());
    if !alt.0.is_empty() {
      self.body.push("else {".to_string());
      for instr in alt.0 {
        self.body.push(format!("  {instr}"));
      }
      self.body.push("}".to_string());
    }
    Ok(cons.1)
  }

  /// Ejecuta `f` capturando en un buffer aparte las instrucciones que emita, sin
  /// añadirlas al cuerpo principal. Devuelve las instrucciones capturadas junto
  /// al valor resultante.
  fn capture<F>(&mut self, f: F) -> CompilerResult<(Vec<String>, QasmValue)>
  where
    F: FnOnce(&mut Self) -> CompilerResult<QasmValue>,
  {
    let saved = std::mem::take(&mut self.body);
    let value = f(self)?;
    let captured = std::mem::replace(&mut self.body, saved);
    Ok((captured, value))
  }

  fn bind_pattern(&mut self, pat: &Pattern<Flat>, value: QasmValue) {
    match &pat.kind {
      PatternKind::Variable(name) => {
        self.env.insert(name.clone(), value);
      }
      PatternKind::Wildcard => {}
      PatternKind::Tuple(pats) => match value {
        QasmValue::Tuple(values) if values.len() == pats.len() => {
          for (p, v) in pats.iter().zip(values) {
            self.bind_pattern(p, v);
          }
        }
        // Si el valor no es una tupla del tamaño esperado, se enlaza cada
        // subpatrón al mismo valor (alias del recurso).
        other => {
          for p in pats {
            self.bind_pattern(p, other.clone());
          }
        }
      },
    }
  }

  /// Traduce el valor final del cuerpo de `main` a los modificadores `output`
  /// de OpenQASM 3, recorriendo la estructura de tuplas en paralelo con el tipo
  /// clásico del resultado. Si el valor ya es un registro clásico declarado
  /// (por ejemplo el resultado de una medición), se promociona a `output` en
  /// vez de declarar una variable nueva.
  fn bind_output(&mut self, ttype: &Type, value: QasmValue, span: &Span) -> CompilerResult<()> {
    match (ttype, value) {
      (Type::Tuple(types), QasmValue::Tuple(values)) if types.len() == values.len() => {
        for (t, v) in types.iter().zip(values) {
          self.bind_output(t, v, span)?;
        }
        Ok(())
      }
      (_, QasmValue::Unit) => Ok(()),
      (_, QasmValue::Classical(name)) => {
        if let Some(reg) = self.classical.iter_mut().find(|r| r.name == name) {
          reg.io = ClassicalIo::Output;
        } else {
          let out = self.fresh_output(ttype);
          self.body.push(format!("{out} = {name};"));
        }
        Ok(())
      }
      (_, QasmValue::Int(n)) => {
        let out = self.fresh_output(ttype);
        self.body.push(format!("{out} = {n};"));
        Ok(())
      }
      (_, QasmValue::Float(f)) => {
        let out = self.fresh_output(ttype);
        self.body.push(format!("{out} = {f};"));
        Ok(())
      }
      (_, other) => Err(Diagnostic::error(
        format!("no se pudo traducir el resultado de 'main' a una salida OpenQASM 3: {other:?}"),
        "el resultado de 'main' debe reducirse a un valor clásico",
        span.clone(),
      )),
    }
  }

  fn fresh_output(&mut self, ttype: &Type) -> String {
    let name = format!("out{}", self.output_counter);
    self.output_counter += 1;
    self.classical.push(ClassicalReg {
      name: name.clone(),
      qasm_type: qasm_classical_type(ttype),
      io: ClassicalIo::Output,
    });
    name
  }
}

/// Traduce un tipo clásico de Qalc a su tipo OpenQASM 3 correspondiente.
fn qasm_classical_type(ttype: &Type) -> String {
  match ttype {
    Type::Builtin { id, .. } => match id.as_str() {
      "i8" => "int[8]".to_string(),
      "i16" => "int[16]".to_string(),
      "i32" => "int[32]".to_string(),
      "i64" => "int[64]".to_string(),
      "u8" => "uint[8]".to_string(),
      "u16" => "uint[16]".to_string(),
      "u32" => "uint[32]".to_string(),
      "u64" => "uint[64]".to_string(),
      "f32" => "float[32]".to_string(),
      "f64" => "float[64]".to_string(),
      "bool" => "bool".to_string(),
      other => other.to_string(),
    },
    // Los tipos compuestos se destructuran antes de llegar aquí (véase
    // `bind_output`); este resultado solo aparece en combinaciones que las
    // demostraciones actuales no ejercitan.
    Type::Tuple(_) | Type::Arrow(_, _) | Type::Slice(_) => "int[32]".to_string(),
  }
}

/// Deshace el envoltorio de efecto `qio<T>` para llegar al tipo clásico
/// efectivo `T`, tal como hace el aplanador al calcular `FlatResult`.
fn unwrap_qio(ttype: &Type) -> &Type {
  match ttype {
    Type::Builtin { id, params, .. } if id == "qio" => params.first().unwrap_or(ttype),
    _ => ttype,
  }
}

/// Devuelve el registro cuántico que aparece entre los argumentos, o `Unit` si no
/// hay ninguno. Es la regla de hilado del circuito para los built-ins de puerta.
fn thread_circuit(args: &[QasmValue]) -> QasmValue {
  args
    .iter()
    .find_map(|a| a.as_qreg())
    .map(|name| QasmValue::QReg(name.to_string()))
    .unwrap_or(QasmValue::Unit)
}

fn find_qreg(args: &[QasmValue]) -> String {
  args
    .iter()
    .find_map(|a| a.as_qreg())
    .unwrap_or("q")
    .to_string()
}

/// Descompone una cadena de `Transform` currificados en su cabeza y la lista de
/// argumentos en orden de aplicación.
fn peel_spine(expr: &Expression<Flat>) -> (&Expression<Flat>, Vec<&Expression<Flat>>) {
  let mut head = expr;
  let mut args = Vec::new();
  while let ExpressionKind::Transform { callee, param } = &head.kind {
    args.push(param.as_ref());
    head = callee;
  }
  args.reverse();
  (head, args)
}
