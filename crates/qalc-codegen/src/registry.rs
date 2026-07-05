//! Registro de emisores de OpenQASM 3 para los built-ins de puerta. Cada puerta
//! básica tiene asociada una plantilla de emisión (`OpenQasm3Transpilable`) que,
//! a partir de los argumentos ya aplanados, produce la instrucción estándar
//! correspondiente. Los built-ins que necesitan contexto de emisión adicional
//! (medición, operaciones de alto nivel que reservan registros) no viven aquí,
//! sino que el generador de código los resuelve directamente con su estado.

use std::collections::HashMap;

use crate::backend::OpenQasm3Transpilable;
use crate::value::QasmValue;

/// Construye la tabla `nombre -> emisor` con las puertas básicas.
pub fn registry() -> HashMap<&'static str, OpenQasm3Transpilable> {
  let mut table: HashMap<&'static str, OpenQasm3Transpilable> = HashMap::new();
  // Puertas de un solo cúbit aplicadas al circuito completo.
  table.insert("H", OpenQasm3Transpilable { emit: emit_h });
  table.insert("X", OpenQasm3Transpilable { emit: emit_x });
  table.insert("Y", OpenQasm3Transpilable { emit: emit_y });
  table.insert("Z", OpenQasm3Transpilable { emit: emit_z });
  // Aplicación posicional y puertas controladas.
  table.insert("at", OpenQasm3Transpilable { emit: emit_at });
  table.insert(
    "controlled",
    OpenQasm3Transpilable {
      emit: emit_controlled,
    },
  );
  table.insert("toffoli", OpenQasm3Transpilable { emit: emit_toffoli });
  // Rotaciones.
  table.insert("rx", OpenQasm3Transpilable { emit: emit_rx });
  table.insert("ry", OpenQasm3Transpilable { emit: emit_ry });
  table.insert("rz", OpenQasm3Transpilable { emit: emit_rz });
  table
}

// --- Utilidades para localizar argumentos por su rol ---

fn qreg(args: &[QasmValue]) -> String {
  args
    .iter()
    .find_map(|a| a.as_qreg())
    .unwrap_or("q")
    .to_string()
}

fn indices(args: &[QasmValue]) -> Vec<usize> {
  args.iter().filter_map(|a| a.as_index()).collect()
}

fn gate_name(args: &[QasmValue]) -> String {
  args
    .iter()
    .find_map(|a| a.as_gate())
    .unwrap_or("x")
    .to_string()
}

/// Fragmento de expresión OpenQASM 3 para el ángulo de una rotación. Puede ser
/// un literal conocido en tiempo de compilación o el nombre de una entrada
/// clásica (`input`), lo que permite parametrizar el circuito con un ángulo
/// que se fija en tiempo de ejecución.
fn angle(args: &[QasmValue]) -> String {
  args
    .iter()
    .find_map(|a| match a {
      QasmValue::Float(f) => Some(f.to_string()),
      QasmValue::Int(n) => Some(n.to_string()),
      QasmValue::Classical(name) => Some(name.clone()),
      _ => None,
    })
    .unwrap_or_else(|| "0".to_string())
}

/// Instrucción estándar de una puerta de un cúbit (`H` -> `h`).
fn gate_instr(name: &str) -> String {
  name.to_lowercase()
}

/// Instrucción estándar de una puerta controlada (`X` -> `cx`).
fn controlled_instr(name: &str) -> String {
  format!("c{}", name.to_lowercase())
}

// --- Emisores ---

fn emit_broadcast(gate: &str, args: &[QasmValue]) -> String {
  format!("{} {};", gate, qreg(args))
}

fn emit_h(args: &[QasmValue]) -> String {
  emit_broadcast("h", args)
}
fn emit_x(args: &[QasmValue]) -> String {
  emit_broadcast("x", args)
}
fn emit_y(args: &[QasmValue]) -> String {
  emit_broadcast("y", args)
}
fn emit_z(args: &[QasmValue]) -> String {
  emit_broadcast("z", args)
}

fn emit_at(args: &[QasmValue]) -> String {
  let q = qreg(args);
  let idx = indices(args).first().copied().unwrap_or(0);
  let gate = gate_instr(&gate_name(args));
  format!("{gate} {q}[{idx}];")
}

fn emit_controlled(args: &[QasmValue]) -> String {
  let q = qreg(args);
  let idxs = indices(args);
  let control = idxs.first().copied().unwrap_or(0);
  let target = idxs.get(1).copied().unwrap_or(1);
  let instr = controlled_instr(&gate_name(args));
  format!("{instr} {q}[{control}], {q}[{target}];")
}

fn emit_toffoli(args: &[QasmValue]) -> String {
  let q = qreg(args);
  let idxs = indices(args);
  let c0 = idxs.first().copied().unwrap_or(0);
  let c1 = idxs.get(1).copied().unwrap_or(1);
  let target = idxs.get(2).copied().unwrap_or(2);
  let gate = gate_name(args);
  // La puerta Toffoli es una `X` doblemente controlada (`ccx`).
  let instr = if gate == "X" {
    "ccx".to_string()
  } else {
    format!("cc{}", gate.to_lowercase())
  };
  format!("{instr} {q}[{c0}], {q}[{c1}], {q}[{target}];")
}

fn emit_rotation(gate: &str, args: &[QasmValue]) -> String {
  format!("{}({}) {};", gate, angle(args), qreg(args))
}

fn emit_rx(args: &[QasmValue]) -> String {
  emit_rotation("rx", args)
}
fn emit_ry(args: &[QasmValue]) -> String {
  emit_rotation("ry", args)
}
fn emit_rz(args: &[QasmValue]) -> String {
  emit_rotation("rz", args)
}
