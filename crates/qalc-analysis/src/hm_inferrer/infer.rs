use std::rc::Rc;

use qalc_ast::{
  Spanned,
  base_ast::{
    Declaration, DeclarationKind, Expression, ExpressionKind, FloatSuffix, NumberSuffix, Pattern,
    PatternKind, SliceExpression, TypeExpression, TypeExpressionKind,
  },
  scope::{Scope, SymbolInfo, SymbolKind},
  typed_ast::{InferenceMetadata, Inferred, TypeWithUnknow},
};
use qalc_utils::{CompilerResult, Diagnostic, Span, gen_id};

#[derive(Debug, Clone)]
pub struct TypeConstraint(pub TypeWithUnknow, pub TypeWithUnknow);

fn infer_pattern(
  pattern: Pattern<Spanned>,
  scope: &Rc<Scope>,
) -> (Pattern<Inferred>, TypeWithUnknow) {
  let span = pattern.meta;

  match pattern.kind {
    PatternKind::Variable(name) => {
      let ttype = TypeWithUnknow::Unknow(gen_id());
      scope.define(SymbolInfo {
        name: name.clone(),
        kind: SymbolKind::Parameter,
        infer_type: ttype.clone(),
        span: span.clone(),
      });

      (
        Pattern {
          kind: PatternKind::Variable(name),
          meta: span,
        },
        ttype,
      )
    }
    PatternKind::Wildcard => (
      Pattern {
        kind: PatternKind::Wildcard,
        meta: span,
      },
      TypeWithUnknow::Unknow(gen_id()),
    ),
    PatternKind::Tuple(patterns) => {
      let mut inferred_pats = Vec::new();
      let mut types = Vec::new();
      for pattern in patterns {
        let (pattern_node, pattern_type) = infer_pattern(pattern, scope);
        inferred_pats.push(pattern_node);
        types.push(pattern_type);
      }

      (
        Pattern {
          kind: PatternKind::Tuple(inferred_pats),
          meta: span,
        },
        TypeWithUnknow::Tuple(types),
      )
    }
  }
}

fn infer_let_block(
  strict: bool,
  assigments: Vec<(Pattern<Spanned>, Expression<Spanned>)>,
  value: Expression<Spanned>,
  scope: &Rc<Scope>,
  span: Span,
) -> CompilerResult<(Expression<Inferred>, Vec<TypeConstraint>)> {
  let let_scope = Rc::new(Scope::new(Some(Rc::clone(scope))));
  let mut constraints = Vec::new();
  let mut inferred_assigments = Vec::new();

  for (pattern, expr) in assigments {
    let (inferred_expr, expr_constraints) = infer_expr(expr, scope)?;
    constraints.extend(expr_constraints);

    let (inferred_pattern, pattern_type) = infer_pattern(pattern, &let_scope);
    constraints.push(TypeConstraint(
      inferred_expr.meta.ttype.clone(),
      pattern_type,
    ));

    inferred_assigments.push((inferred_pattern, inferred_expr));
  }

  let (inferred_value, value_constraints) = infer_expr(value, &let_scope)?;
  constraints.extend(value_constraints);
  let ttype = inferred_value.meta.ttype.clone();

  Ok((
    Expression {
      kind: ExpressionKind::LetBlock {
        strict,
        assigments: inferred_assigments,
        value: Box::new(inferred_value),
      },
      meta: InferenceMetadata { span, ttype },
    },
    constraints,
  ))
}

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
  let lambda_scope = Rc::new(Scope::new(Some(Rc::clone(scope))));

  let param_types: Vec<TypeWithUnknow> = params
    .iter()
    .map(|name| {
      let ttype = TypeWithUnknow::Unknow(gen_id());
      lambda_scope.define(SymbolInfo {
        name: name.clone(),
        kind: SymbolKind::Parameter,
        infer_type: ttype.clone(),
        span: span.clone(),
      });

      ttype
    })
    .collect();

  let (body, constraints) = infer_expr(body, &lambda_scope)?;

  let mut ttype = body.meta.ttype.clone();
  for param_type in param_types.into_iter().rev() {
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

fn infer_transform(
  callee: Expression<Spanned>,
  param: Expression<Spanned>,
  scope: &Rc<Scope>,
  span: Span,
) -> CompilerResult<(Expression<Inferred>, Vec<TypeConstraint>)> {
  let return_type = TypeWithUnknow::Unknow(gen_id());
  let (callee_node, callee_constraints) = infer_expr(callee, scope)?;
  let (param_node, param_constraints) = infer_expr(param, scope)?;

  let arrow_type = TypeWithUnknow::Arrow(
    Box::new(param_node.meta.ttype.clone()),
    Box::new(return_type.clone()),
  );

  let mut constraints = vec![TypeConstraint(callee_node.meta.ttype.clone(), arrow_type)];
  constraints.extend(callee_constraints);
  constraints.extend(param_constraints);

  Ok((
    Expression {
      kind: ExpressionKind::Transform {
        callee: Box::new(callee_node),
        param: Box::new(param_node),
      },
      meta: InferenceMetadata {
        span,
        ttype: return_type,
      },
    },
    constraints,
  ))
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

fn infer_expr(
  expr: Expression<Spanned>,
  scope: &Rc<Scope>,
) -> CompilerResult<(Expression<Inferred>, Vec<TypeConstraint>)> {
  let span = expr.meta;
  match expr.kind {
    ExpressionKind::LetBlock {
      strict,
      assigments,
      value,
    } => infer_let_block(strict, assigments, *value, scope, span),
    ExpressionKind::IfElseBlock {
      test,
      consequent,
      alternative,
    } => infer_ifelse(*test, *consequent, *alternative, scope, span),
    ExpressionKind::Transform { callee, param } => infer_transform(*callee, *param, scope, span),
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

pub fn type_expr_to_infer_type(te: TypeExpression<Spanned>) -> TypeWithUnknow {
  match te.kind {
    TypeExpressionKind::Identifier(id) => TypeWithUnknow::Builtin {
      id,
      params: vec![],
      linear: false,
    },
    TypeExpressionKind::IdentifierWithParams { identifier, params } => TypeWithUnknow::Builtin {
      id: identifier,
      params: params.into_iter().map(type_expr_to_infer_type).collect(),
      linear: false,
    },
    TypeExpressionKind::Slice(inner) => {
      TypeWithUnknow::Slice(Box::new(type_expr_to_infer_type(*inner)))
    }
    TypeExpressionKind::Tuple(types) => {
      TypeWithUnknow::Tuple(types.into_iter().map(type_expr_to_infer_type).collect())
    }
    TypeExpressionKind::Transform {
      input_type,
      output_type,
    } => TypeWithUnknow::Arrow(
      Box::new(type_expr_to_infer_type(*input_type)),
      Box::new(type_expr_to_infer_type(*output_type)),
    ),
  }
}

fn remap_type_expr(te: TypeExpression<Spanned>) -> TypeExpression<Inferred> {
  TypeExpression {
    meta: te.meta,
    kind: match te.kind {
      TypeExpressionKind::Identifier(id) => TypeExpressionKind::Identifier(id),
      TypeExpressionKind::IdentifierWithParams { identifier, params } => {
        TypeExpressionKind::IdentifierWithParams {
          identifier,
          params: params.into_iter().map(remap_type_expr).collect(),
        }
      }
      TypeExpressionKind::Slice(inner) => {
        TypeExpressionKind::Slice(Box::new(remap_type_expr(*inner)))
      }
      TypeExpressionKind::Tuple(types) => {
        TypeExpressionKind::Tuple(types.into_iter().map(remap_type_expr).collect())
      }
      TypeExpressionKind::Transform {
        input_type,
        output_type,
      } => TypeExpressionKind::Transform {
        input_type: Box::new(remap_type_expr(*input_type)),
        output_type: Box::new(remap_type_expr(*output_type)),
      },
    },
  }
}

pub fn infer_decl(
  decl: Declaration<Spanned>,
  declared_type: TypeWithUnknow,
  scope: &Rc<Scope>,
) -> CompilerResult<(Declaration<Inferred>, Vec<TypeConstraint>)> {
  let span = decl.meta;
  match decl.kind {
    DeclarationKind::Function {
      identifier,
      type_annotation,
      params,
      body,
    } => {
      let fn_scope = Rc::new(Scope::new(Some(Rc::clone(scope))));

      let param_types: Vec<TypeWithUnknow> = params
        .iter()
        .map(|name| {
          let ttype = TypeWithUnknow::Unknow(gen_id());
          fn_scope.define(SymbolInfo {
            name: name.clone(),
            kind: SymbolKind::Parameter,
            infer_type: ttype.clone(),
            span: span.clone(),
          });

          ttype
        })
        .collect();

      let (inferred_body, mut constraints) = infer_expr(body, &fn_scope)?;

      let mut inferred_type = inferred_body.meta.ttype.clone();
      for param_type in param_types.into_iter().rev() {
        inferred_type = TypeWithUnknow::Arrow(Box::new(param_type), Box::new(inferred_type));
      }

      constraints.push(TypeConstraint(inferred_type.clone(), declared_type));

      Ok((
        Declaration {
          kind: DeclarationKind::Function {
            identifier,
            type_annotation: remap_type_expr(type_annotation),
            params,
            body: inferred_body,
          },
          meta: InferenceMetadata {
            span,
            ttype: inferred_type,
          },
        },
        constraints,
      ))
    }
    DeclarationKind::Type { .. } => Err(Diagnostic::error(
      "Type declarations are not supported in the MVP.",
      "Feature outside MVP — only function declarations are supported.",
      span,
    )),
  }
}
