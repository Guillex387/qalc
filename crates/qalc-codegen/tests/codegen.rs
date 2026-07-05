use std::rc::Rc;

use qalc_analysis::{HMInferrer, Inliner, Normalizer, RecursionChecker};
use qalc_builtins::standard_builtins;
use qalc_codegen::Codegen;
use qalc_parser::parse_program;
use qalc_utils::{CompilerStep, Source};

/// Conduce el pipeline completo sobre un fuente y devuelve el texto OpenQASM 3.
fn compile(src: &str) -> String {
  let source = Rc::new(Source::memory(src.to_string()));
  let concrete = parse_program(&source).expect("parse");
  let mut diags = vec![];
  let normalized = Normalizer::default()
    .transform(concrete, &mut diags)
    .expect("normalize");
  let normalized = RecursionChecker::default()
    .transform(normalized, &mut diags)
    .expect("recursion");
  let typed = HMInferrer::new(standard_builtins())
    .transform(normalized, &mut diags)
    .expect("infer");
  let flat = Inliner::default()
    .transform(typed, &mut diags)
    .expect("inline");
  Codegen::default()
    .transform(flat, &mut diags)
    .expect("codegen")
}

#[test]
fn bell_circuit_emits_h_and_cnot() {
  // El circuito se construye con el built-in `qcirc` dentro del cuerpo, ya que
  // 'main' solo puede recibir y devolver tipos clásicos.
  let qasm = compile("main :: qio<u8>\nmain = qcirc 2 |> at 0 H |> controlled 0 1 X |> measure;\n");
  eprintln!("{qasm}");

  assert!(
    qasm.contains("OPENQASM 3.0;"),
    "falta la cabecera de versión"
  );
  assert!(
    qasm.contains("qubit[2] q0;"),
    "el registro debe tener anchura 2"
  );
  assert!(qasm.contains("h q0[0];"), "falta la puerta H sobre q0[0]");
  assert!(
    qasm.contains("cx q0[0], q0[1];"),
    "falta la CNOT q0[0]->q0[1]"
  );

  // El orden importa: primero la H, luego la CNOT.
  let h_pos = qasm.find("h q0[0];").unwrap();
  let cx_pos = qasm.find("cx q0[0], q0[1];").unwrap();
  assert!(h_pos < cx_pos, "la H debe emitirse antes que la CNOT");
}

#[test]
fn rotation_and_direct_gate() {
  let qasm = compile("main :: qio<u8>\nmain = qcirc 3 |> H |> at 2 X |> measure;\n");
  eprintln!("{qasm}");
  assert!(
    qasm.contains("qubit[3] q0;"),
    "el tamaño pasado al constructor fija la anchura"
  );
  assert!(
    qasm.contains("h q0;"),
    "H directa se aplica a todo el registro"
  );
  assert!(qasm.contains("x q0[2];"), "falta la X sobre q0[2]");
}

#[test]
fn parametric_circuit_uses_input_and_output_modifiers() {
  // El ángulo de rotación es un parámetro clásico de 'main': debe traducirse a
  // una entrada `input` de OpenQASM 3, permitiendo ejecutar el mismo circuito
  // con distintos ángulos sin recompilar.
  let qasm = compile(
    "main :: f64 -> qio<u8>\nmain theta = qcirc 2 |> at 0 H |> controlled 0 1 X |> rz theta |> measure;\n",
  );
  eprintln!("{qasm}");

  assert!(
    qasm.contains("input float[64] p0;"),
    "el ángulo clásico debe declararse como entrada"
  );
  assert!(
    qasm.contains("rz(p0) q0;"),
    "la rotación debe referenciar la entrada, no una constante"
  );
  assert!(
    qasm.contains("output bit[2] c0;"),
    "el resultado de la medición debe promocionarse a salida"
  );
}

#[test]
fn measurement_emits_classical_bit() {
  let qasm = compile("main :: qio<u8>\nmain = measure (qcirc 2 |> at 0 H);\n");
  eprintln!("{qasm}");
  assert!(qasm.contains("h q0[0];"), "falta la H previa a la medición");
  assert!(
    qasm.contains("output bit[2] c0;"),
    "el resultado medido debe declararse como salida"
  );
  assert!(qasm.contains("c0 = measure q0;"), "falta la medición");
}

#[test]
fn qcirc_with_non_constant_size_is_rejected() {
  // El tamaño de un `qcirc` debe conocerse en tiempo de compilación, ya que fija
  // la anchura de la declaración `qubit[N]`.
  let source = Rc::new(Source::memory(
    "main :: i32 -> qio<u8>\nmain n = measure (qcirc n |> H);\n".to_string(),
  ));
  let concrete = parse_program(&source).expect("parse");
  let mut diags = vec![];
  let normalized = Normalizer::default()
    .transform(concrete, &mut diags)
    .expect("normalize");
  let normalized = RecursionChecker::default()
    .transform(normalized, &mut diags)
    .expect("recursion");
  let typed = HMInferrer::new(standard_builtins())
    .transform(normalized, &mut diags)
    .expect("infer");
  let flat = Inliner::default()
    .transform(typed, &mut diags)
    .expect("inline");
  assert!(Codegen::default().transform(flat, &mut diags).is_err());
}

#[test]
fn grover_2bit_emits_real_adder_and_comparator_gates() {
  // El sumador y el comparador deben traducirse a puertas Toffoli/CNOT reales
  // que cruzan registros (no a stubs vacíos como en los built-ins eliminados
  // qPowConst/qEqConst).
  let qasm = compile(include_str!("../../../examples/grover_2bit.qalc"));
  eprintln!("{qasm}");

  assert!(qasm.contains("OPENQASM 3.0;"));
  // El registro de búsqueda (2 cúbits) puesto en superposición.
  assert!(
    qasm.contains("qubit[2] q0;"),
    "falta el registro de búsqueda"
  );
  assert!(
    qasm.contains("h q0;"),
    "el registro de búsqueda debe pasar por H"
  );
  // Al menos una puerta Toffoli/CNOT cruzando dos registros distintos
  // (control y objetivo con nombres de registro diferentes).
  assert!(
    qasm.contains("ccx q0[0], q1[0], q4[0];"),
    "falta un Toffoli del sumador cruzando registros"
  );
  assert!(
    qasm.contains("cx q0[0], q3[0];"),
    "falta un CNOT del sumador cruzando registros"
  );
  // El difusor de Grover (H, X, CZ, X, H) sobre el registro de búsqueda.
  assert!(
    qasm.contains("cz q0[0], q0[1];"),
    "falta la Z controlada del difusor"
  );
  // La medición final se promociona a salida.
  assert!(
    qasm.contains("output bit[2] c0;"),
    "el resultado medido debe promocionarse a salida"
  );
  assert!(qasm.contains("c0 = measure q0;"), "falta la medición final");
}
