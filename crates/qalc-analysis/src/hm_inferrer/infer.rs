use std::rc::Rc;

use qalc_ast::{
  Spanned,
  base_ast::{Expression, ExpressionKind, FloatSuffix, NumberSuffix, SliceExpression},
  scope::Scope,
  typed_ast::TypeWithUnknow,
};
use qalc_utils::{CompilerResult, Diagnostic, Span, gen_id};

#[derive(Debug, Clone)]
pub struct TypeConstraint(TypeWithUnknow, TypeWithUnknow);

// TODO: instead of infer the type, transform the AST node to a typed node

fn infer_ifelse(
  consequent: &Expression<Spanned>,
  alternative: &Expression<Spanned>,
  scope: &Rc<Scope>,
) -> CompilerResult<(TypeWithUnknow, Vec<TypeConstraint>)> {
  let return_type = TypeWithUnknow::Unknow(gen_id());
  let (consequent_type, consequent_constraints) = infer_expr(consequent, scope)?;
  let (alternative_type, alternative_constraints) = infer_expr(alternative, scope)?;
  let block_contraints = vec![
    TypeConstraint(return_type.clone(), consequent_type),
    TypeConstraint(return_type.clone(), alternative_type),
  ];
  let constraints = vec![
    block_contraints,
    consequent_constraints,
    alternative_constraints,
  ]
  .concat();

  Ok((return_type, constraints))
}

fn infer_tuple(
  exprs: &Vec<Expression<Spanned>>,
  scope: &Rc<Scope>,
) -> CompilerResult<(TypeWithUnknow, Vec<TypeConstraint>)> {
  let mut tuple_internal_types: Vec<TypeWithUnknow> = Vec::new();
  let mut constraints: Vec<TypeConstraint> = Vec::new();
  for expr in exprs {
    let (internal_type, mut internal_constraints) = infer_expr(expr, scope)?;
    tuple_internal_types.push(internal_type);
    constraints.append(&mut internal_constraints);
  }

  Ok((TypeWithUnknow::Tuple(tuple_internal_types), constraints))
}

fn infer_slice(
  slice_expr: &SliceExpression<Spanned>,
  scope: &Rc<Scope>,
) -> CompilerResult<(TypeWithUnknow, Vec<TypeConstraint>)> {
  match slice_expr {
    SliceExpression::Items(items) => {
      if items.is_empty() {
        return Ok((TypeWithUnknow::Unknow(gen_id()), vec![]));
      }
      let mut item_constraint = Vec::new();
      let mut inherit_constraints: Vec<TypeConstraint> = Vec::new();
      let slice_internal_type = TypeWithUnknow::Unknow(gen_id());

      for item in items {
        let (ttype, mut internal_constraint) = infer_expr(item, scope)?;
        item_constraint.push(TypeConstraint(slice_internal_type.clone(), ttype));
        inherit_constraints.append(&mut internal_constraint);
      }

      let constraints = vec![item_constraint, inherit_constraints].concat();

      Ok((
        TypeWithUnknow::Slice(Box::new(slice_internal_type)),
        constraints,
      ))
    }
    SliceExpression::Range(start, end) => {
      let slice_internal_type = TypeWithUnknow::Unknow(gen_id());

      let (start_type, start_constraints) = infer_expr(start, scope)?;
      let (end_type, end_constraints) = infer_expr(end, scope)?;
      let items_constraints = vec![
        TypeConstraint(slice_internal_type.clone(), start_type),
        TypeConstraint(slice_internal_type.clone(), end_type),
      ];
      let constraints = vec![items_constraints, start_constraints, end_constraints].concat();

      Ok((
        TypeWithUnknow::Slice(Box::new(slice_internal_type)),
        constraints,
      ))
    }
  }
}

fn infer_identifier(ident: &str, scope: &Rc<Scope>, span: Span) -> CompilerResult<TypeWithUnknow> {
  match scope.resolve(ident) {
    Some(symbol_info) => Ok(symbol_info.infer_type),
    None => {
      return Err(Diagnostic::error(
        format!("Undefined variable: {}", ident),
        "Undefined variable — the name is not defined in this scope.",
        span,
      ));
    }
  }
}

fn infer_integer(suffix: &NumberSuffix) -> TypeWithUnknow {
  match suffix {
    NumberSuffix::I8 => TypeWithUnknow::Builtin {
      id: "i8".to_string(),
      params: vec![],
      linear: false,
    },
    NumberSuffix::I16 => TypeWithUnknow::Builtin {
      id: "i16".to_string(),
      params: vec![],
      linear: false,
    },
    NumberSuffix::I32 => TypeWithUnknow::Builtin {
      id: "i32".to_string(),
      params: vec![],
      linear: false,
    },
    NumberSuffix::I64 => TypeWithUnknow::Builtin {
      id: "i64".to_string(),
      params: vec![],
      linear: false,
    },
    NumberSuffix::U8 => TypeWithUnknow::Builtin {
      id: "u8".to_string(),
      params: vec![],
      linear: false,
    },
    NumberSuffix::U16 => TypeWithUnknow::Builtin {
      id: "u16".to_string(),
      params: vec![],
      linear: false,
    },
    NumberSuffix::U32 => TypeWithUnknow::Builtin {
      id: "u32".to_string(),
      params: vec![],
      linear: false,
    },
    NumberSuffix::U64 => TypeWithUnknow::Builtin {
      id: "u64".to_string(),
      params: vec![],
      linear: false,
    },
    NumberSuffix::Unknow => TypeWithUnknow::Builtin {
      id: "i32".to_string(),
      params: vec![],
      linear: false,
    },
  }
}

fn infer_float(suffix: &FloatSuffix) -> TypeWithUnknow {
  match &suffix {
    FloatSuffix::F32 => TypeWithUnknow::Builtin {
      id: "f32".to_string(),
      params: vec![],
      linear: false,
    },
    FloatSuffix::F64 => TypeWithUnknow::Builtin {
      id: "f64".to_string(),
      params: vec![],
      linear: false,
    },
    FloatSuffix::Unknow => TypeWithUnknow::Builtin {
      id: "f64".to_string(),
      params: vec![],
      linear: false,
    },
  }
}

pub fn infer_expr(
  expr: &Expression<Spanned>,
  scope: &Rc<Scope>,
) -> CompilerResult<(TypeWithUnknow, Vec<TypeConstraint>)> {
  match &expr.kind {
    ExpressionKind::LetBlock {
      strict,
      assigments,
      value,
    } => todo!(),
    ExpressionKind::IfElseBlock {
      test: _,
      consequent,
      alternative,
    } => infer_ifelse(consequent, alternative, scope),
    ExpressionKind::Transform { callee, param } => todo!(),
    ExpressionKind::Lambda { params, body } => todo!(),
    ExpressionKind::Tuple(expressions) => infer_tuple(expressions, scope),
    ExpressionKind::Slice(slice_expr) => infer_slice(slice_expr, scope),
    ExpressionKind::Identifier(ident) => {
      infer_identifier(ident, scope, expr.meta.clone()).map(|ttype| (ttype, vec![]))
    }
    ExpressionKind::IntegerLiteral { number: _, suffix } => Ok((infer_integer(suffix), vec![])),
    ExpressionKind::FloatLiteral { number: _, suffix } => Ok((infer_float(suffix), vec![])),
  }
}
