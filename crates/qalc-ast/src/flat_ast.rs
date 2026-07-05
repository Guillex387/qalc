use std::rc::Rc;

use qalc_utils::Span;

use crate::ASTMetadata;
use crate::base_ast::Expression;
use crate::scope::Scope;
use crate::typed_ast::Type;

/// Marca sobre una expresión del FlatAST para señalarla como "especial".
/// Hoy sólo distingue los lambdas que sobreviven al inlining (los que son
/// argumento de un builtin de orden superior). Es extensible si aparecen
/// otros casos que el backend deba tratar de forma diferenciada.
#[derive(Debug, Clone, PartialEq)]
pub enum FlatMark {
  Normal,
  ResidualLambda,
}

/// Metadata del FlatAST: como `TypeMetadata` pero sin `scope` (span + tipo
/// resuelto) más una `FlatMark`. El scope por-nodo es redundante en un programa
/// ya aplanado; el scope global se conserva en la raíz (`FlatAST::global_scope`).
#[derive(Debug, Clone)]
pub struct FlatMetadata {
  pub span: Span,
  pub ttype: Type,
  pub mark: FlatMark,
}

/// Marcador `ASTMetadata` del FlatAST. Reutiliza la expresión base
/// (`Expression<Flat>`), que conserva la variante `Lambda`.
#[derive(Debug, Clone)]
pub struct Flat;

impl ASTMetadata for Flat {
  type DeclarationMeta = FlatMetadata;
  type ExpressionMeta = FlatMetadata;
  type PatternMeta = Span;
  type TypeMeta = Span;
}

/// Uno de los `n` parámetros de entrada del programa aplanado (vienen de los
/// parámetros de `main`).
#[derive(Debug, Clone)]
pub struct FlatParam {
  pub name: String,
  pub ttype: Type,
}

/// Descripción de los `m` resultados del programa aplanado (salen del tipo de
/// retorno de `main`): cuántos son y si vienen envueltos en un `qio`.
#[derive(Debug, Clone)]
pub struct FlatResult {
  pub arity: usize,
  pub qio: bool,
}

/// Programa aplanado: una única expresión (el cuerpo de `main` con todas las
/// llamadas no built-in inlinadas), sus `n` parámetros y sus `m` resultados.
#[derive(Debug, Clone)]
pub struct FlatAST {
  pub params: Vec<FlatParam>,
  pub body: Expression<Flat>,
  pub result: FlatResult,
  pub global_scope: Rc<Scope>,
}
