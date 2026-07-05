//! Valores que circulan durante la generación de código. Sustituyen al
//! `FlatValue` del esbozo de la memoria, adaptándose a los tipos reales del
//! `FlatAST`. Un `QasmValue` es la forma ya reducida de un argumento de
//! built-in que un emisor puede traducir directamente a OpenQASM 3.

/// Valor resuelto de una subexpresión del `FlatAST` durante la emisión.
#[derive(Debug, Clone, PartialEq)]
pub enum QasmValue {
  /// Referencia a un registro cuántico completo (por ejemplo `q`).
  QReg(String),
  /// Índice de un cúbit concreto dentro del registro.
  Qubit(usize),
  /// Literal entero, empleado como índice de cúbit o como constante clásica.
  Int(i128),
  /// Literal en coma flotante, empleado como ángulo de rotación.
  Float(f64),
  /// Nombre de una puerta pasada como valor a un combinador de orden superior
  /// (por ejemplo la `H` de `at 0 H`).
  Gate(String),
  /// Fragmento clásico, como un bit de resultado de medición.
  Classical(String),
  /// Tupla de valores (resultado de un nodo `Tuple` del `FlatAST`).
  Tuple(Vec<QasmValue>),
  /// Ausencia de valor, resultado de una operación de solo efecto.
  Unit,
}

impl QasmValue {
  /// Devuelve el nombre del registro si el valor referencia un circuito.
  pub fn as_qreg(&self) -> Option<&str> {
    match self {
      QasmValue::QReg(name) => Some(name),
      _ => None,
    }
  }

  /// Devuelve el índice entero si el valor es un literal entero o un cúbit.
  pub fn as_index(&self) -> Option<usize> {
    match self {
      QasmValue::Int(n) => usize::try_from(*n).ok(),
      QasmValue::Qubit(i) => Some(*i),
      _ => None,
    }
  }

  /// Devuelve el nombre de la puerta si el valor la representa.
  pub fn as_gate(&self) -> Option<&str> {
    match self {
      QasmValue::Gate(name) => Some(name),
      _ => None,
    }
  }
}
