//! Comprobación de consumo único de recursos lineales
//! (`report/07-capítulo-backend.tex`, sección `sec:linear-checker`).
//!
//! Recorre la única expresión del `FlatAST` producido por el aplanador y cuenta,
//! para cada identificador de tipo lineal en el ámbito, sus apariciones. Un
//! recurso que termina su ámbito con cero apariciones es una fuga (un cúbit que
//! nunca se midió ni se liberó); con más de una, una duplicación (clonar
//! información cuántica, prohibido por el teorema de no clonación). Es una
//! validación pura, `FlatAST -> FlatAST`, análoga en espíritu al comprobador de
//! recursión.
//!
//! El conteo es sintáctico sobre el árbol ya aplanado, donde cada recurso lineal
//! aparece directamente como argumento concreto de una aplicación de built-in.
//! Un recurso referenciado dentro de una lambda residual (thunk) se cuenta en el
//! punto en que la lambda se construye. La implementación es experimental y no
//! cubre todas las combinaciones de thunks anidados.

use std::collections::HashMap;

use qalc_ast::base_ast::{Expression, ExpressionKind, Pattern, PatternKind, SliceExpression};
use qalc_ast::flat_ast::{Flat, FlatAST};
use qalc_ast::typed_ast::Type;
use qalc_utils::{CompilerResult, CompilerStep, Diagnostic, Span};

/// Multiconjunto de apariciones libres de identificadores en una expresión.
type Uses = HashMap<String, usize>;

#[derive(Default)]
pub struct LinearChecker;

impl CompilerStep for LinearChecker {
  type Input = FlatAST;
  type Output = FlatAST;

  fn transform(&self, input: FlatAST, _: &mut Vec<Diagnostic>) -> CompilerResult<FlatAST> {
    // Recorre el cuerpo validando las ligaduras lineales internas y devuelve las
    // apariciones libres restantes (las de los parámetros de entrada).
    let uses = check_expr(&input.body)?;

    for param in &input.params {
      if param.ttype.is_linear() {
        let count = uses.get(&param.name).copied().unwrap_or(0);
        ensure_single_use(&param.name, count, input.body.meta.span.clone())?;
      }
    }

    Ok(input)
  }
}

/// Recorre una expresión, valida las ligaduras lineales que introduce y devuelve
/// el multiconjunto de apariciones libres respecto al ámbito exterior.
fn check_expr(expr: &Expression<Flat>) -> CompilerResult<Uses> {
  match &expr.kind {
    ExpressionKind::Identifier(name) => {
      let mut uses = Uses::new();
      *uses.entry(name.clone()).or_default() += 1;
      Ok(uses)
    }
    ExpressionKind::IntegerLiteral { .. } | ExpressionKind::FloatLiteral { .. } => Ok(Uses::new()),
    ExpressionKind::Transform { .. } => {
      // Los argumentos directos de una única aplicación (p. ej.
      // `crossToffoli tc 0 tc 1 rc 0`) pueden referenciar el mismo recurso más
      // de una vez sin que sea una duplicación: son ángulos distintos de una
      // misma operación atómica (aquí, dos cúbits distintos del mismo
      // registro), no dos consumos independientes que puedan divergir. Se
      // combinan con el máximo por identificador, igual que las ramas de un
      // `if`, en vez de sumarlos.
      let (head, args) = peel_spine(expr);
      let mut combined = check_expr(head)?;
      for arg in args {
        combined = branch_max(combined, check_expr(arg)?);
      }
      Ok(combined)
    }
    ExpressionKind::Tuple(exprs) => {
      let mut acc = Uses::new();
      for e in exprs {
        acc = merge(acc, check_expr(e)?);
      }
      Ok(acc)
    }
    ExpressionKind::Slice(slice) => match slice {
      SliceExpression::Items(items) => {
        let mut acc = Uses::new();
        for e in items {
          acc = merge(acc, check_expr(e)?);
        }
        Ok(acc)
      }
      SliceExpression::Range(start, end) => Ok(merge(check_expr(start)?, check_expr(end)?)),
    },
    ExpressionKind::Lambda { params, body } => {
      // Regla del thunk: las apariciones dentro de la lambda cuentan en el punto
      // de construcción, descontando los parámetros que la propia lambda liga.
      let mut uses = check_expr(body)?;
      for p in params {
        uses.remove(p);
      }
      Ok(uses)
    }
    ExpressionKind::IfElseBlock {
      test,
      consequent,
      alternative,
    } => {
      // Solo se ejecuta una rama, así que las apariciones en las ramas se toman
      // como el máximo por identificador, no como su suma.
      let branches = branch_max(check_expr(consequent)?, check_expr(alternative)?);
      Ok(merge(check_expr(test)?, branches))
    }
    ExpressionKind::LetBlock {
      assigments, value, ..
    } => {
      let mut acc = check_expr(value)?;
      // Se procesan las ligaduras en orden inverso, de modo que `acc` contiene
      // siempre las apariciones del ámbito posterior a la ligadura actual.
      for (pat, rhs) in assigments.iter().rev() {
        for (name, ttype, span) in pattern_var_types(pat, &rhs.meta.ttype) {
          if ttype.is_linear() {
            let count = acc.get(&name).copied().unwrap_or(0);
            ensure_single_use(&name, count, span)?;
          }
          acc.remove(&name);
        }
        acc = merge(acc, check_expr(rhs)?);
      }
      Ok(acc)
    }
  }
}

/// Comprueba que un recurso lineal se consume exactamente una vez.
fn ensure_single_use(name: &str, count: usize, span: Span) -> CompilerResult<()> {
  match count {
    1 => Ok(()),
    0 => Err(Diagnostic::error(
      format!("el recurso lineal '{name}' nunca se consume (fuga)"),
      "este recurso cuántico debe usarse exactamente una vez",
      span,
    )),
    n => Err(Diagnostic::error(
      format!("el recurso lineal '{name}' se usa {n} veces (duplicación)"),
      "un recurso cuántico no puede clonarse ni consumirse más de una vez",
      span,
    )),
  }
}

/// Empareja los patrones de una ligadura con el tipo de su valor para deducir el
/// tipo (y por tanto la linealidad) de cada variable que introduce.
fn pattern_var_types(pat: &Pattern<Flat>, ttype: &Type) -> Vec<(String, Type, Span)> {
  match &pat.kind {
    PatternKind::Variable(name) => vec![(name.clone(), ttype.clone(), pat.meta.clone())],
    PatternKind::Wildcard => vec![],
    PatternKind::Tuple(pats) => match ttype {
      Type::Tuple(types) if types.len() == pats.len() => pats
        .iter()
        .zip(types)
        .flat_map(|(p, t)| pattern_var_types(p, t))
        .collect(),
      // Si no se puede emparejar la estructura, se atribuye el tipo completo a
      // cada subpatrón (aproximación conservadora).
      _ => pats
        .iter()
        .flat_map(|p| pattern_var_types(p, ttype))
        .collect(),
    },
  }
}

/// Suma dos multiconjuntos de apariciones.
fn merge(mut a: Uses, b: Uses) -> Uses {
  for (name, count) in b {
    *a.entry(name).or_default() += count;
  }
  a
}

/// Máximo por identificador de dos multiconjuntos (para las ramas de un `if`).
fn branch_max(mut a: Uses, b: Uses) -> Uses {
  for (name, count) in b {
    let entry = a.entry(name).or_default();
    *entry = (*entry).max(count);
  }
  a
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

#[cfg(test)]
mod tests {
  use std::rc::Rc;

  use qalc_builtins::standard_builtins;
  use qalc_parser::parse_program;
  use qalc_utils::{CompilerStep, Source};

  use super::LinearChecker;
  use crate::{HMInferrer, Inliner, Normalizer, RecursionChecker};

  fn check(src: &str) -> bool {
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
    LinearChecker::default().transform(flat, &mut diags).is_ok()
  }

  #[test]
  fn bell_consumes_circuit_once() {
    // El circuito se construye con el built-in `qcirc` (ya no como parámetro de
    // 'main', que debe ser clásico) y se consume una única vez en la cadena.
    assert!(check(
      "main :: qio<u8>\nmain = qcirc 2 |> at 0 H |> controlled 0 1 X |> measure;\n"
    ));
  }

  #[test]
  fn unused_linear_resource_is_a_leak() {
    assert!(!check("main :: i32\nmain = let c = qcirc 2 in 0i32;\n"));
  }

  #[test]
  fn duplicated_linear_resource_is_rejected() {
    assert!(!check(
      "main :: (qio<u8>, qio<u8>)\nmain = let c = qcirc 2 in (measure (H c), measure (H c));\n"
    ));
  }

  #[test]
  fn classical_param_is_not_constrained() {
    // Un parámetro clásico puede ignorarse sin que sea una fuga.
    assert!(check("main :: i32 -> i32\nmain n = 0i32;\n"));
  }

  #[test]
  fn grover_2bit_example_passes_linear_check() {
    // Sumador, comparador, oráculo y grover encadenan `qu8`/`qbool` a través de
    // varias funciones y bloques `linear`; cada recurso (a, b, target, sum,
    // los acarreos, isMatch) debe consumirse exactamente una vez.
    let src = include_str!("../../../../examples/grover_2bit.qalc");
    assert!(check(src));
  }
}
