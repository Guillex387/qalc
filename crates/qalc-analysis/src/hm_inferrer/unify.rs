use std::collections::HashMap;

use qalc_ast::typed_ast::TypeWithUnknow;
use qalc_utils::{CompilerResult, Diagnostic, Span};

use super::infer::TypeConstraint;

pub fn unify(constraints: Vec<TypeConstraint>) -> CompilerResult<HashMap<usize, TypeWithUnknow>> {
  let mut subst = HashMap::new();
  for TypeConstraint { lhs, rhs, span } in constraints {
    let lhs = apply(&subst, lhs);
    let rhs = apply(&subst, rhs);
    unify_one(lhs, rhs, span, &mut subst)?;
  }
  Ok(subst)
}

pub fn apply(subst: &HashMap<usize, TypeWithUnknow>, ty: TypeWithUnknow) -> TypeWithUnknow {
  match ty {
    TypeWithUnknow::Unknow(id) => match subst.get(&id) {
      Some(resolved) => apply(subst, resolved.clone()),
      None => TypeWithUnknow::Unknow(id),
    },
    TypeWithUnknow::Arrow(a, b) => TypeWithUnknow::Arrow(
      Box::new(apply(subst, *a)),
      Box::new(apply(subst, *b)),
    ),
    TypeWithUnknow::Tuple(ts) => {
      TypeWithUnknow::Tuple(ts.into_iter().map(|t| apply(subst, t)).collect())
    }
    TypeWithUnknow::Slice(t) => TypeWithUnknow::Slice(Box::new(apply(subst, *t))),
    TypeWithUnknow::Builtin { id, params, linear } => TypeWithUnknow::Builtin {
      id,
      params: params.into_iter().map(|p| apply(subst, p)).collect(),
      linear,
    },
  }
}

fn unify_one(
  lhs: TypeWithUnknow,
  rhs: TypeWithUnknow,
  span: Span,
  subst: &mut HashMap<usize, TypeWithUnknow>,
) -> CompilerResult<()> {
  if lhs == rhs {
    return Ok(());
  }
  match (lhs, rhs) {
    (TypeWithUnknow::Unknow(id), ty) => {
      if occurs(id, &ty) {
        return Err(Diagnostic::error(
          "Infinite type",
          "This type would be infinite — a type variable cannot appear in its own definition.",
          span,
        ));
      }
      subst.insert(id, ty);
      Ok(())
    }
    (ty, TypeWithUnknow::Unknow(id)) => {
      if occurs(id, &ty) {
        return Err(Diagnostic::error(
          "Infinite type",
          "This type would be infinite — a type variable cannot appear in its own definition.",
          span,
        ));
      }
      subst.insert(id, ty);
      Ok(())
    }
    (TypeWithUnknow::Arrow(a1, a2), TypeWithUnknow::Arrow(b1, b2)) => {
      unify_one(*a1, *b1, span.clone(), subst)?;
      let a2 = apply(subst, *a2);
      let b2 = apply(subst, *b2);
      unify_one(a2, b2, span, subst)
    }
    (TypeWithUnknow::Tuple(a_types), TypeWithUnknow::Tuple(b_types))
      if a_types.len() == b_types.len() =>
    {
      for (a, b) in a_types.into_iter().zip(b_types) {
        let a = apply(subst, a);
        let b = apply(subst, b);
        unify_one(a, b, span.clone(), subst)?;
      }
      Ok(())
    }
    (TypeWithUnknow::Slice(a), TypeWithUnknow::Slice(b)) => unify_one(*a, *b, span, subst),
    (
      TypeWithUnknow::Builtin {
        id: id_a,
        params: pa,
        linear: la,
      },
      TypeWithUnknow::Builtin {
        id: id_b,
        params: pb,
        linear: lb,
      },
    ) if id_a == id_b && la == lb && pa.len() == pb.len() => {
      for (a, b) in pa.into_iter().zip(pb) {
        let a = apply(subst, a);
        let b = apply(subst, b);
        unify_one(a, b, span.clone(), subst)?;
      }
      Ok(())
    }
    (lhs, rhs) => Err(Diagnostic::error(
      format!("Type mismatch: `{:?}` is not compatible with `{:?}`", lhs, rhs),
      "These types cannot be unified.",
      span,
    )),
  }
}

fn occurs(id: usize, ty: &TypeWithUnknow) -> bool {
  match ty {
    TypeWithUnknow::Unknow(other_id) => id == *other_id,
    TypeWithUnknow::Arrow(a, b) => occurs(id, a) || occurs(id, b),
    TypeWithUnknow::Tuple(ts) => ts.iter().any(|t| occurs(id, t)),
    TypeWithUnknow::Slice(t) => occurs(id, t),
    TypeWithUnknow::Builtin { params, .. } => params.iter().any(|p| occurs(id, p)),
  }
}
