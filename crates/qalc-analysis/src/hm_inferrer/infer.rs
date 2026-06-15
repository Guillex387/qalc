use std::rc::Rc;

use qalc_ast::{
  Spanned,
  base_ast::{Expression, ExpressionKind, FloatSuffix, NumberSuffix, SliceExpression},
  scope::Scope,
  typed_ast::{InferenceMetadata, Inferred, TypeWithUnknow},
};
use qalc_utils::{CompilerResult, Diagnostic, Span, gen_id};

#[derive(Debug, Clone)]
pub struct TypeConstraint(pub TypeWithUnknow, pub TypeWithUnknow);

fn infer_ifelse(
  test: Expression<Spanned>,
  consequent: Expression<Spanned>,
  alternative: Expression<Spanned>,
  scope: &Rc<Scope>,
  span: Span,
) -> CompilerResult<(Expression<Inferred>, Vec<TypeConstraint>)> {
  let return_type = TypeWithUnknow::Unknow(gen_id());
  let (test_node, test_constraints) = infer_expr(test, scope)?;
  let (consequent_node, consequent_constraints) = infer_expr(consequent, scope)?;
  let (alternative_node, alternative_constraints) = infer_expr(alternative, scope)?;

  let mut constraints = vec![
    TypeConstraint(return_type.clone(), consequent_node.meta.ttype.clone()),
    TypeConstraint(return_type.clone(), alternative_node.meta.ttype.clone()),
  ];
  constraints.extend(consequent_constraints);
  constraints.extend(alternative_constraints);
  constraints.extend(test_constraints);

  Ok((
    Expression {
      kind: ExpressionKind::IfElseBlock {
        test: Box::new(test_node),
        consequent: Box::new(consequent_node),
        alternative: Box::new(alternative_node),
      },
      meta: InferenceMetadata {
        span,
        ttype: return_type,
      },
    },
    constraints,
  ))
}

fn infer_lambda(
  params: Vec<String>,
  body: Expression<Spanned>,
  scope: &Rc<Scope>,
  span: Span,
) -> CompilerResult<(Expression<Inferred>, Vec<TypeConstraint>)> {
  let (body, constraints) = infer_expr(body, scope)?;
  let mut ttype = body.meta.ttype.clone();
  for _ in 0..params.len() {
    let param_type = TypeWithUnknow::Unknow(gen_id());
    ttype = TypeWithUnknow::Arrow(Box::new(param_type), Box::new(ttype));
  }

  Ok((
    Expression {
      kind: ExpressionKind::Lambda {
        params,
        body: Box::new(body),
      },
      meta: InferenceMetadata { span, ttype },
    },
    constraints,
  ))
}

fn infer_tuple(
  exprs: Vec<Expression<Spanned>>,
  scope: &Rc<Scope>,
  span: Span,
) -> CompilerResult<(Expression<Inferred>, Vec<TypeConstraint>)> {
  let mut tuple_types: Vec<TypeWithUnknow> = Vec::new();
  let mut constraints: Vec<TypeConstraint> = Vec::new();
  let mut inferred_exprs: Vec<Expression<Inferred>> = Vec::new();

  for expr in exprs {
    let (node, node_constraints) = infer_expr(expr, scope)?;
    tuple_types.push(node.meta.ttype.clone());
    constraints.extend(node_constraints);
    inferred_exprs.push(node);
  }

  Ok((
    Expression {
      kind: ExpressionKind::Tuple(inferred_exprs),
      meta: InferenceMetadata {
        span,
        ttype: TypeWithUnknow::Tuple(tuple_types),
      },
    },
    constraints,
  ))
}

fn infer_slice(
  slice_expr: SliceExpression<Spanned>,
  scope: &Rc<Scope>,
  span: Span,
) -> CompilerResult<(Expression<Inferred>, Vec<TypeConstraint>)> {
  match slice_expr {
    SliceExpression::Items(items) => {
      if items.is_empty() {
        let ttype = TypeWithUnknow::Slice(Box::new(TypeWithUnknow::Unknow(gen_id())));
        return Ok((
          Expression {
            kind: ExpressionKind::Slice(SliceExpression::Items(vec![])),
            meta: InferenceMetadata { span, ttype },
          },
          vec![],
        ));
      }

      let slice_internal_type = TypeWithUnknow::Unknow(gen_id());
      let mut item_constraints: Vec<TypeConstraint> = Vec::new();
      let mut inherited_constraints: Vec<TypeConstraint> = Vec::new();
      let mut inferred_items: Vec<Expression<Inferred>> = Vec::new();

      for item in items {
        let (node, node_constraints) = infer_expr(item, scope)?;
        item_constraints.push(TypeConstraint(
          slice_internal_type.clone(),
          node.meta.ttype.clone(),
        ));
        inherited_constraints.extend(node_constraints);
        inferred_items.push(node);
      }

      let mut constraints = item_constraints;
      constraints.extend(inherited_constraints);

      Ok((
        Expression {
          kind: ExpressionKind::Slice(SliceExpression::Items(inferred_items)),
          meta: InferenceMetadata {
            span,
            ttype: TypeWithUnknow::Slice(Box::new(slice_internal_type)),
          },
        },
        constraints,
      ))
    }
    SliceExpression::Range(start, end) => {
      let slice_internal_type = TypeWithUnknow::Unknow(gen_id());
      let (start_node, start_constraints) = infer_expr(*start, scope)?;
      let (end_node, end_constraints) = infer_expr(*end, scope)?;

      let mut constraints = vec![
        TypeConstraint(slice_internal_type.clone(), start_node.meta.ttype.clone()),
        TypeConstraint(slice_internal_type.clone(), end_node.meta.ttype.clone()),
      ];
      constraints.extend(start_constraints);
      constraints.extend(end_constraints);

      Ok((
        Expression {
          kind: ExpressionKind::Slice(SliceExpression::Range(
            Box::new(start_node),
            Box::new(end_node),
          )),
          meta: InferenceMetadata {
            span,
            ttype: TypeWithUnknow::Slice(Box::new(slice_internal_type)),
          },
        },
        constraints,
      ))
    }
  }
}

fn infer_identifier(
  ident: String,
  scope: &Rc<Scope>,
  span: Span,
) -> CompilerResult<(Expression<Inferred>, Vec<TypeConstraint>)> {
  match scope.resolve(ident.as_str()) {
    Some(symbol_info) => Ok((
      Expression {
        kind: ExpressionKind::Identifier(ident.to_string()),
        meta: InferenceMetadata {
          span,
          ttype: symbol_info.infer_type,
        },
      },
      vec![],
    )),
    None => Err(Diagnostic::error(
      format!("Undefined variable: {}", ident),
      "Undefined variable — the name is not defined in this scope.",
      span,
    )),
  }
}

fn infer_integer(suffix: NumberSuffix) -> TypeWithUnknow {
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

fn infer_float(suffix: FloatSuffix) -> TypeWithUnknow {
  match suffix {
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
  expr: Expression<Spanned>,
  scope: &Rc<Scope>,
) -> CompilerResult<(Expression<Inferred>, Vec<TypeConstraint>)> {
  let span = expr.meta;
  match expr.kind {
    ExpressionKind::LetBlock { .. } => todo!(),
    ExpressionKind::IfElseBlock {
      test,
      consequent,
      alternative,
    } => infer_ifelse(*test, *consequent, *alternative, scope, span),
    ExpressionKind::Transform { .. } => todo!(),
    ExpressionKind::Lambda { params, body } => infer_lambda(params, *body, scope, span),
    ExpressionKind::Tuple(expressions) => infer_tuple(expressions, scope, span),
    ExpressionKind::Slice(slice_expr) => infer_slice(slice_expr, scope, span),
    ExpressionKind::Identifier(ident) => infer_identifier(ident, scope, span),
    ExpressionKind::IntegerLiteral { number, suffix } => Ok((
      Expression {
        kind: ExpressionKind::IntegerLiteral {
          number: number,
          suffix: suffix.clone(),
        },
        meta: InferenceMetadata {
          span,
          ttype: infer_integer(suffix),
        },
      },
      vec![],
    )),
    ExpressionKind::FloatLiteral { number, suffix } => Ok((
      Expression {
        kind: ExpressionKind::FloatLiteral {
          number: number,
          suffix: suffix.clone(),
        },
        meta: InferenceMetadata {
          span,
          ttype: infer_float(suffix),
        },
      },
      vec![],
    )),
  }
}
