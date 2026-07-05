//! Abstracción de backend descrita en el capítulo de backend de la memoria
//! (`report/07-capítulo-backend.tex`, sección `sec:codegen`). `Backend` es un
//! rasgo marcador que identifica un formato de salida; `Transpilable<B>` se
//! parametriza sobre él para que un mismo built-in pueda emitir a formatos
//! distintos sin compartir tipo. Añadir un backend nuevo (por ejemplo Quil) no
//! exige tocar estas definiciones, solo aportar una implementación de
//! `Transpilable` por built-in.
//!
//! Diverge del esbozo de la memoria en que `transpile` recibe `&[QasmValue]`
//! (los tipos reales del `FlatAST`) en lugar del `FlatValue` conceptual.

use crate::value::QasmValue;

/// Rasgo marcador de un formato de salida soportado por el compilador.
pub trait Backend {}

/// El backend actual, OpenQASM 3.
pub struct OpenQasm3;
impl Backend for OpenQasm3 {}

/// Ejemplo de un segundo backend hipotético (el ensamblado cuántico de Rigetti)
/// que ilustra la extensibilidad de la arquitectura.
pub struct Quil;
impl Backend for Quil {}

/// Traducción de un built-in a un backend concreto. Su único método recibe los
/// argumentos ya aplanados del built-in y devuelve el fragmento de código que lo
/// representa.
pub trait Transpilable<B: Backend> {
  fn transpile(&self, args: &[QasmValue]) -> String;
}

/// Implementación concreta que usa el backend de OpenQASM 3. Envuelve la función
/// Rust que compone el fragmento de salida para un built-in en particular.
pub struct OpenQasm3Transpilable {
  pub emit: fn(&[QasmValue]) -> String,
}

impl Transpilable<OpenQasm3> for OpenQasm3Transpilable {
  fn transpile(&self, args: &[QasmValue]) -> String {
    (self.emit)(args)
  }
}
