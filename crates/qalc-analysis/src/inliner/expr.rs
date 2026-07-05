use std::collections::{HashMap, HashSet};

use qalc_ast::base_ast::{Expression, ExpressionKind, Pattern, PatternKind, SliceExpression};
use qalc_ast::flat_ast::{Flat, FlatMark, FlatMetadata};
use qalc_ast::typed_ast::{TypeMetadata, Typed};
use qalc_utils::{CompilerResult, Diagnostic, gen_id};

use super::FnDef;

#[derive(Clone)]
pub(super) struct LocalLambda {
  params: Vec<String>,
  body: Expression<Typed>,
}

type Lambdas = HashMap<String, LocalLambda>;

fn flat_meta(meta: TypeMetadata, mark: FlatMark) -> FlatMetadata {
  FlatMetadata {
    span: meta.span,
    ttype: meta.ttype,
    mark,
  }
}

fn convert_pattern(pat: Pattern<Typed>) -> Pattern<Flat> {
  Pattern {
    meta: pat.meta,
    kind: match pat.kind {
      PatternKind::Variable(name) => PatternKind::Variable(name),
      PatternKind::Wildcard => PatternKind::Wildcard,
      PatternKind::Tuple(pats) => {
        PatternKind::Tuple(pats.into_iter().map(convert_pattern).collect())
      }
    },
  }
}

fn pattern_vars(pat: &Pattern<Typed>) -> Vec<String> {
  match &pat.kind {
    PatternKind::Variable(name) => vec![name.clone()],
    PatternKind::Tuple(pats) => pats.iter().flat_map(pattern_vars).collect(),
    PatternKind::Wildcard => vec![],
  }
}

fn pattern_binds(pat: &Pattern<Typed>, name: &str) -> bool {
  pattern_vars(pat).iter().any(|v| v == name)
}

fn as_local_lambda(e: &Expression<Typed>, lambdas: &Lambdas) -> Option<LocalLambda> {
  match &e.kind {
    ExpressionKind::Lambda { params, body } => Some(LocalLambda {
      params: params.clone(),
      body: (**body).clone(),
    }),
    ExpressionKind::Identifier(name) => lambdas.get(name).cloned(),
    _ => None,
  }
}

enum TransformClass {
  Reducible,
  Curry,
  Other,
}

fn classify_transform(
  callee: &Expression<Typed>,
  funcs: &HashMap<String, FnDef>,
  shadowed: &HashSet<String>,
  lambdas: &Lambdas,
) -> TransformClass {
  let mut head = callee;
  let mut arg_count = 1;
  while let ExpressionKind::Transform { callee: inner, .. } = &head.kind {
    arg_count += 1;
    head = inner;
  }
  let arity_class = |arity: usize| {
    if arity == arg_count {
      TransformClass::Reducible
    } else {
      TransformClass::Curry
    }
  };
  match &head.kind {
    ExpressionKind::Identifier(name) if lambdas.contains_key(name) => {
      arity_class(lambdas[name].params.len())
    }
    ExpressionKind::Identifier(name) if funcs.contains_key(name) && !shadowed.contains(name) => {
      arity_class(funcs[name].params.len())
    }
    ExpressionKind::Lambda { params, .. } => arity_class(params.len()),
    _ => TransformClass::Other,
  }
}

fn peel_spine(
  callee: Box<Expression<Typed>>,
  param: Expression<Typed>,
) -> (Expression<Typed>, Vec<Expression<Typed>>) {
  let mut args = vec![param];
  let mut head = *callee;
  while let ExpressionKind::Transform {
    callee: inner,
    param: arg,
  } = head.kind
  {
    args.push(*arg);
    head = *inner;
  }
  args.reverse();
  (head, args)
}

pub(super) fn inline_expr(
  expr: Expression<Typed>,
  funcs: &HashMap<String, FnDef>,
  shadowed: &HashSet<String>,
  lambdas: &Lambdas,
) -> CompilerResult<Expression<Flat>> {
  let Expression { kind, meta } = expr;
  match kind {
    ExpressionKind::Transform { callee, param } => {
      match classify_transform(&callee, funcs, shadowed, lambdas) {
        TransformClass::Other => Ok(Expression {
          kind: ExpressionKind::Transform {
            callee: Box::new(inline_expr(*callee, funcs, shadowed, lambdas)?),
            param: Box::new(inline_expr(*param, funcs, shadowed, lambdas)?),
          },
          meta: flat_meta(meta, FlatMark::Normal),
        }),
        TransformClass::Curry => Err(Diagnostic::error(
          "aplicación parcial o sobre-aplicación de función no soportada todavía",
          "la currificación está fuera de alcance por ahora",
          meta.span,
        )),
        TransformClass::Reducible => {
          let (head, args) = peel_spine(callee, *param);
          let (params, body) = match head.kind {
            ExpressionKind::Identifier(name) => {
              if let Some(local) = lambdas.get(&name) {
                (local.params.clone(), local.body.clone())
              } else {
                let f = &funcs[&name];
                (f.params.clone(), f.body.clone())
              }
            }
            ExpressionKind::Lambda { params, body } => (params, *body),
            _ => unreachable!("classify_transform garantiza una cabeza reducible"),
          };
          reduce(params, body, args, funcs, shadowed, lambdas, meta)
        }
      }
    }
    ExpressionKind::Lambda { params, body } => {
      let mut inner_shadowed = shadowed.clone();
      let mut inner_lambdas = lambdas.clone();
      for p in &params {
        inner_shadowed.insert(p.clone());
        inner_lambdas.remove(p);
      }
      let body = Box::new(inline_expr(*body, funcs, &inner_shadowed, &inner_lambdas)?);
      Ok(Expression {
        kind: ExpressionKind::Lambda { params, body },
        meta: flat_meta(meta, FlatMark::ResidualLambda),
      })
    }
    ExpressionKind::Identifier(name) => {
      if let Some(local) = lambdas.get(&name) {
        let lambda_expr = Expression {
          kind: ExpressionKind::Lambda {
            params: local.params.clone(),
            body: Box::new(local.body.clone()),
          },
          meta,
        };
        inline_expr(lambda_expr, funcs, shadowed, lambdas)
      } else if funcs.contains_key(&name) && !shadowed.contains(&name) {
        let f = &funcs[&name];
        if f.params.is_empty() {
          inline_expr(f.body.clone(), funcs, shadowed, lambdas)
        } else {
          Err(Diagnostic::error(
            format!(
              "referencia a la función '{name}' sin aplicar (orden superior) no soportada todavía"
            ),
            "sólo se soportan llamadas completamente aplicadas por ahora",
            meta.span,
          ))
        }
      } else {
        Ok(Expression {
          kind: ExpressionKind::Identifier(name),
          meta: flat_meta(meta, FlatMark::Normal),
        })
      }
    }
    ExpressionKind::LetBlock {
      strict,
      assigments,
      value,
    } => {
      let mut inner_shadowed = shadowed.clone();
      let mut inner_lambdas = lambdas.clone();
      let mut new_assigments = Vec::new();
      for (pat, e) in assigments {
        if let PatternKind::Variable(name) = &pat.kind {
          if let Some(local) = as_local_lambda(&e, &inner_lambdas) {
            inner_lambdas.insert(name.clone(), local);
            continue;
          }
        }
        let flat_e = inline_expr(e, funcs, &inner_shadowed, &inner_lambdas)?;
        for v in pattern_vars(&pat) {
          inner_shadowed.insert(v.clone());
          inner_lambdas.remove(&v);
        }
        new_assigments.push((convert_pattern(pat), flat_e));
      }
      let value = inline_expr(*value, funcs, &inner_shadowed, &inner_lambdas)?;
      if new_assigments.is_empty() {
        Ok(value)
      } else {
        Ok(Expression {
          kind: ExpressionKind::LetBlock {
            strict,
            assigments: new_assigments,
            value: Box::new(value),
          },
          meta: flat_meta(meta, FlatMark::Normal),
        })
      }
    }
    ExpressionKind::IfElseBlock {
      test,
      consequent,
      alternative,
    } => Ok(Expression {
      kind: ExpressionKind::IfElseBlock {
        test: Box::new(inline_expr(*test, funcs, shadowed, lambdas)?),
        consequent: Box::new(inline_expr(*consequent, funcs, shadowed, lambdas)?),
        alternative: Box::new(inline_expr(*alternative, funcs, shadowed, lambdas)?),
      },
      meta: flat_meta(meta, FlatMark::Normal),
    }),
    ExpressionKind::Tuple(exprs) => Ok(Expression {
      kind: ExpressionKind::Tuple(
        exprs
          .into_iter()
          .map(|e| inline_expr(e, funcs, shadowed, lambdas))
          .collect::<CompilerResult<_>>()?,
      ),
      meta: flat_meta(meta, FlatMark::Normal),
    }),
    ExpressionKind::Slice(slice) => {
      let slice = match slice {
        SliceExpression::Items(items) => SliceExpression::Items(
          items
            .into_iter()
            .map(|e| inline_expr(e, funcs, shadowed, lambdas))
            .collect::<CompilerResult<_>>()?,
        ),
        SliceExpression::Range(start, end) => SliceExpression::Range(
          Box::new(inline_expr(*start, funcs, shadowed, lambdas)?),
          Box::new(inline_expr(*end, funcs, shadowed, lambdas)?),
        ),
      };
      Ok(Expression {
        kind: ExpressionKind::Slice(slice),
        meta: flat_meta(meta, FlatMark::Normal),
      })
    }
    ExpressionKind::IntegerLiteral { number, suffix } => Ok(Expression {
      kind: ExpressionKind::IntegerLiteral { number, suffix },
      meta: flat_meta(meta, FlatMark::Normal),
    }),
    ExpressionKind::FloatLiteral { number, suffix } => Ok(Expression {
      kind: ExpressionKind::FloatLiteral { number, suffix },
      meta: flat_meta(meta, FlatMark::Normal),
    }),
  }
}

fn reduce(
  params: Vec<String>,
  body: Expression<Typed>,
  args: Vec<Expression<Typed>>,
  funcs: &HashMap<String, FnDef>,
  shadowed: &HashSet<String>,
  lambdas: &Lambdas,
  result_meta: TypeMetadata,
) -> CompilerResult<Expression<Flat>> {
  let mut renamed_body = body;
  let mut inner_shadowed = shadowed.clone();
  let mut inner_lambdas = lambdas.clone();
  let mut assigments = Vec::new();
  for (param, arg) in params.into_iter().zip(args) {
    let fresh = format!("{param}_{}", gen_id());
    renamed_body = rename_in_expr(renamed_body, &param, &fresh);

    let arg_is_lambda = matches!(arg.kind, ExpressionKind::Lambda { .. });
    let arg_lambda_local = match &arg.kind {
      ExpressionKind::Identifier(x) => lambdas.get(x).cloned(),
      _ => None,
    };

    if arg_is_lambda {
      if let ExpressionKind::Lambda {
        params: lp,
        body: lb,
      } = arg.kind
      {
        inner_lambdas.insert(
          fresh.clone(),
          LocalLambda {
            params: lp,
            body: *lb,
          },
        );
      }
    } else if let Some(local) = arg_lambda_local {
      inner_lambdas.insert(fresh.clone(), local);
    } else {
      let flat_arg = inline_expr(arg, funcs, shadowed, lambdas)?;
      let span = flat_arg.meta.span.clone();
      assigments.push((
        Pattern {
          kind: PatternKind::Variable(fresh.clone()),
          meta: span,
        },
        flat_arg,
      ));
      inner_shadowed.insert(fresh.clone());
      inner_lambdas.remove(&fresh);
    }
  }
  let value = inline_expr(renamed_body, funcs, &inner_shadowed, &inner_lambdas)?;
  if assigments.is_empty() {
    Ok(value)
  } else {
    Ok(Expression {
      kind: ExpressionKind::LetBlock {
        strict: false,
        assigments,
        value: Box::new(value),
      },
      meta: flat_meta(result_meta, FlatMark::Normal),
    })
  }
}

fn rename_in_expr(expr: Expression<Typed>, from: &str, to: &str) -> Expression<Typed> {
  let Expression { kind, meta } = expr;
  let kind = match kind {
    ExpressionKind::Identifier(name) => {
      ExpressionKind::Identifier(if name == from { to.to_string() } else { name })
    }
    ExpressionKind::Lambda { params, body } => {
      if params.iter().any(|p| p == from) {
        ExpressionKind::Lambda { params, body }
      } else {
        ExpressionKind::Lambda {
          params,
          body: Box::new(rename_in_expr(*body, from, to)),
        }
      }
    }
    ExpressionKind::LetBlock {
      strict,
      assigments,
      value,
    } => {
      let mut shadowed_now = false;
      let mut new_assigments = Vec::new();
      for (pat, e) in assigments {
        let e = if shadowed_now {
          e
        } else {
          rename_in_expr(e, from, to)
        };
        if pattern_binds(&pat, from) {
          shadowed_now = true;
        }
        new_assigments.push((pat, e));
      }
      let value = if shadowed_now {
        value
      } else {
        Box::new(rename_in_expr(*value, from, to))
      };
      ExpressionKind::LetBlock {
        strict,
        assigments: new_assigments,
        value,
      }
    }
    ExpressionKind::Transform { callee, param } => ExpressionKind::Transform {
      callee: Box::new(rename_in_expr(*callee, from, to)),
      param: Box::new(rename_in_expr(*param, from, to)),
    },
    ExpressionKind::IfElseBlock {
      test,
      consequent,
      alternative,
    } => ExpressionKind::IfElseBlock {
      test: Box::new(rename_in_expr(*test, from, to)),
      consequent: Box::new(rename_in_expr(*consequent, from, to)),
      alternative: Box::new(rename_in_expr(*alternative, from, to)),
    },
    ExpressionKind::Tuple(exprs) => ExpressionKind::Tuple(
      exprs
        .into_iter()
        .map(|e| rename_in_expr(e, from, to))
        .collect(),
    ),
    ExpressionKind::Slice(slice) => ExpressionKind::Slice(match slice {
      SliceExpression::Items(items) => SliceExpression::Items(
        items
          .into_iter()
          .map(|e| rename_in_expr(e, from, to))
          .collect(),
      ),
      SliceExpression::Range(start, end) => SliceExpression::Range(
        Box::new(rename_in_expr(*start, from, to)),
        Box::new(rename_in_expr(*end, from, to)),
      ),
    }),
    lit @ (ExpressionKind::IntegerLiteral { .. } | ExpressionKind::FloatLiteral { .. }) => lit,
  };
  Expression { kind, meta }
}
