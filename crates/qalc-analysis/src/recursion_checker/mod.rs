use std::collections::{HashMap, HashSet};

use qalc_ast::{
  Spanned,
  base_ast::{DeclarationKind, Expression, ExpressionKind, Pattern, PatternKind, SliceExpression},
  normalized_ast::NormalizedAST,
};
use qalc_utils::{CompilerResult, Diagnostic, Span};

use qalc_utils::CompilerStep;

#[derive(Debug, PartialEq, Clone)]
enum NodeState {
  Unvisited,
  Visiting,
  Visited,
}

#[derive(Default)]
pub struct RecursionChecker;

impl CompilerStep for RecursionChecker {
  type Input = NormalizedAST;
  type Output = NormalizedAST;

  fn transform(&self, input: NormalizedAST, _: &mut Vec<Diagnostic>) -> CompilerResult<NormalizedAST> {
    let global_fns = collect_global_functions(&input);
    let graph = build_graph(&input, &global_fns);
    detect_cycles(&graph)?;
    Ok(input)
  }
}

fn collect_global_functions(ast: &NormalizedAST) -> HashSet<String> {
  ast
    .declarations
    .iter()
    .filter_map(|decl| {
      if let DeclarationKind::Function { identifier, .. } = &decl.kind {
        Some(identifier.clone())
      } else {
        None
      }
    })
    .collect()
}

fn build_graph(
  ast: &NormalizedAST,
  global_fns: &HashSet<String>,
) -> HashMap<String, Vec<(String, Span)>> {
  let mut graph: HashMap<String, Vec<(String, Span)>> = HashMap::new();

  for decl in &ast.declarations {
    if let DeclarationKind::Function { identifier, params, body, .. } = &decl.kind {
      let mut local_bindings: HashSet<String> = params.iter().cloned().collect();
      let mut deps: Vec<(String, Span)> = Vec::new();
      collect_deps_in_expr(body, global_fns, &mut local_bindings, &mut deps);
      graph.insert(identifier.clone(), deps);
    }
  }

  graph
}

fn collect_deps_in_expr(
  expr: &Expression<Spanned>,
  global_fns: &HashSet<String>,
  local_bindings: &mut HashSet<String>,
  deps: &mut Vec<(String, Span)>,
) {
  match &expr.kind {
    ExpressionKind::Identifier(name) => {
      if global_fns.contains(name) && !local_bindings.contains(name) {
        deps.push((name.clone(), expr.meta.clone()));
      }
    }
    ExpressionKind::Transform { callee, param } => {
      collect_deps_in_expr(callee, global_fns, local_bindings, deps);
      collect_deps_in_expr(param, global_fns, local_bindings, deps);
    }
    ExpressionKind::Lambda { params, body } => {
      let mut lambda_bindings = local_bindings.clone();
      for p in params {
        lambda_bindings.insert(p.clone());
      }
      collect_deps_in_expr(body, global_fns, &mut lambda_bindings, deps);
    }
    ExpressionKind::LetBlock { assigments, value, .. } => {
      let mut let_bindings = local_bindings.clone();
      for (pattern, rhs) in assigments {
        collect_deps_in_expr(rhs, global_fns, &mut let_bindings, deps);
        for var in collect_pattern_vars(pattern) {
          let_bindings.insert(var);
        }
      }
      collect_deps_in_expr(value, global_fns, &mut let_bindings, deps);
    }
    ExpressionKind::IfElseBlock { test, consequent, alternative } => {
      collect_deps_in_expr(test, global_fns, local_bindings, deps);
      collect_deps_in_expr(consequent, global_fns, local_bindings, deps);
      collect_deps_in_expr(alternative, global_fns, local_bindings, deps);
    }
    ExpressionKind::Tuple(exprs) => {
      for e in exprs {
        collect_deps_in_expr(e, global_fns, local_bindings, deps);
      }
    }
    ExpressionKind::Slice(slice) => match slice {
      SliceExpression::Items(items) => {
        for item in items {
          collect_deps_in_expr(item, global_fns, local_bindings, deps);
        }
      }
      SliceExpression::Range(start, end) => {
        collect_deps_in_expr(start, global_fns, local_bindings, deps);
        collect_deps_in_expr(end, global_fns, local_bindings, deps);
      }
    },
    ExpressionKind::IntegerLiteral { .. } | ExpressionKind::FloatLiteral { .. } => {}
  }
}

fn collect_pattern_vars(pattern: &Pattern<Spanned>) -> Vec<String> {
  match &pattern.kind {
    PatternKind::Variable(name) => vec![name.clone()],
    PatternKind::Tuple(pats) => pats.iter().flat_map(collect_pattern_vars).collect(),
    PatternKind::Wildcard => vec![],
  }
}

fn detect_cycles(graph: &HashMap<String, Vec<(String, Span)>>) -> CompilerResult<()> {
  let mut states: HashMap<String, NodeState> =
    graph.keys().map(|k| (k.clone(), NodeState::Unvisited)).collect();

  let function_names: Vec<String> = graph.keys().cloned().collect();
  for name in function_names {
    if states.get(&name) == Some(&NodeState::Unvisited) {
      let mut path: Vec<String> = Vec::new();
      dfs(&name, graph, &mut states, &mut path)?;
    }
  }

  Ok(())
}

fn dfs(
  current: &str,
  graph: &HashMap<String, Vec<(String, Span)>>,
  states: &mut HashMap<String, NodeState>,
  path: &mut Vec<String>,
) -> CompilerResult<()> {
  if states.get(current) == Some(&NodeState::Visiting) {
    let prev = path.last().expect("path must not be empty when a cycle is detected");
    let span = graph[prev]
      .iter()
      .find(|(name, _)| name == current)
      .map(|(_, span)| span.clone())
      .expect("dependency edge must exist when cycle back-edge is found");
    path.push(current.to_string());
    let cycle_start = path.iter().position(|n| n == current).unwrap();
    let cycle_path = path[cycle_start..].join(" -> ");
    return Err(Diagnostic::error(
      format!("Recursion detected: {}", cycle_path),
      "this reference creates a recursive cycle",
      span,
    ));
  }

  if states.get(current) == Some(&NodeState::Visited) {
    return Ok(());
  }

  states.insert(current.to_string(), NodeState::Visiting);
  path.push(current.to_string());

  let neighbors: Vec<(String, Span)> = graph.get(current).cloned().unwrap_or_default();
  for (neighbor, _) in &neighbors {
    dfs(neighbor, graph, states, path)?;
  }

  states.insert(current.to_string(), NodeState::Visited);
  path.pop();

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::rc::Rc;

  use qalc_parser::parse_program;
  use qalc_utils::{CompilerStep, Source};

  use crate::Normalizer;

  use super::RecursionChecker;

  fn check(src: &str) -> bool {
    let source = Rc::new(Source::memory(src.to_string()));
    let concrete = parse_program(&source).expect("parse");
    let normalized = Normalizer.transform(concrete, &mut vec![]).expect("normalize");
    RecursionChecker.transform(normalized, &mut vec![]).is_ok()
  }

  #[test]
  fn test_no_cycle_passes() {
    assert!(check("f :: num  f = 42;  g :: num  g = f;"));
  }

  #[test]
  fn test_direct_recursion_is_rejected() {
    assert!(!check("f :: num -> num  f x = f x;"));
  }

  #[test]
  fn test_indirect_recursion_is_rejected() {
    assert!(!check("f :: num  f = g;  g :: num  g = f;"));
  }

  #[test]
  fn test_shadowed_param_not_treated_as_dep() {
    // param named `f` shadows the global function `f`
    assert!(check("f :: num -> num  f f = 42;"));
  }

  #[test]
  fn test_higher_order_cycle_is_rejected() {
    assert!(!check("h :: num  h = f;  f :: num  f = h;"));
  }

  #[test]
  fn test_lambda_shadowing_not_a_dep() {
    // lambda param named `f` shadows global `f` inside the lambda body
    assert!(check("f :: num  f = 42;  g :: num -> num  g = (f) -> f;"));
  }

  #[test]
  fn test_three_way_indirect_cycle_is_rejected() {
    assert!(!check(
      "f :: num  f = g;  g :: num  g = h;  h :: num  h = f;"
    ));
  }

  #[test]
  fn test_diamond_dependency_no_cycle_passes() {
    // base <- left <- top, base <- right: DAG, no cycle
    assert!(check(
      "base :: num  base = 1;  left :: num  left = base;  right :: num  right = base;  top :: num  top = left;"
    ));
  }

  #[test]
  fn test_chain_dependency_no_cycle_passes() {
    // f -> g -> h: linear chain, no cycle
    assert!(check(
      "h :: num  h = 1;  g :: num  g = h;  f :: num  f = g;"
    ));
  }

  #[test]
  fn test_recursion_in_let_rhs_is_rejected() {
    // recursive reference in the RHS of a let binding
    assert!(!check("f :: num  f = let x = f in 1;"));
  }

  #[test]
  fn test_recursion_in_let_value_is_rejected() {
    // recursive reference in the `in` part of a let block
    assert!(!check("f :: num  f = let x = 1 in f;"));
  }

  #[test]
  fn test_let_binding_shadows_global_no_dep() {
    // `let f = 1 in f` — the `f` inside `in` is the let-bound value, not the global function
    assert!(check("f :: num  f = 42;  g :: num  g = let f = 1 in f;"));
  }

  #[test]
  fn test_recursion_in_if_else_is_rejected() {
    // recursive reference inside an if-else branch
    assert!(!check("f :: num  f = if 1 then f else 0;"));
  }

  #[test]
  fn test_mutual_recursion_through_calls_is_rejected() {
    // f calls g, g calls f — mutual recursion through function application
    assert!(!check(
      "f :: num -> num  f x = g x;  g :: num -> num  g x = f x;"
    ));
  }

  #[test]
  fn test_independent_functions_pass() {
    // multiple functions, none depending on each other
    assert!(check(
      "a :: num  a = 1;  b :: num  b = 2;  c :: num  c = 3;"
    ));
  }
}
