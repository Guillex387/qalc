use std::collections::HashMap;
use std::rc::Rc;

use qalc_ast::{
  base_ast::{
    Declaration, DeclarationKind, Expression, ExpressionKind, Pattern, PatternKind,
    SliceExpression, TypeExpression, TypeExpressionKind,
  },
  scope::Scope,
  typed_ast::{InferenceMetadata, Inferred, Type, TypeMetadata, TypeWithUnknow, Typed},
};
use qalc_utils::{CompilerResult, Diagnostic};

use super::unify::apply;

fn resolve_meta(
  meta: InferenceMetadata,
  subst: &HashMap<usize, TypeWithUnknow>,
) -> CompilerResult<TypeMetadata> {
  let span = meta.span.clone();
  let resolved = apply(subst, meta.ttype);
  let ttype = Type::try_from(resolved).map_err(|_| {
    Diagnostic::error(
      "Ambiguous type",
      "Could not infer a concrete type — add a type annotation.",
      span.clone(),
    )
  })?;
  Ok(TypeMetadata {
    span,
    ttype,
    scope: meta.scope,
  })
}

fn finalize_pattern(pattern: Pattern<Inferred>) -> Pattern<Typed> {
  Pattern {
    meta: pattern.meta,
    kind: match pattern.kind {
      PatternKind::Variable(name) => PatternKind::Variable(name),
      PatternKind::Wildcard => PatternKind::Wildcard,
      PatternKind::Tuple(pats) => {
        PatternKind::Tuple(pats.into_iter().map(finalize_pattern).collect())
      }
    },
  }
}

fn finalize_type_expr(
  te: TypeExpression<Inferred>,
  scope: &Rc<Scope>,
) -> CompilerResult<TypeExpression<Typed>> {
  let span = te.meta;
  let (kind, ttype) = match te.kind {
    TypeExpressionKind::Identifier(id) => {
      let ttype = Type::Builtin {
        id: id.clone(),
        params: vec![],
        linear: false,
      };
      (TypeExpressionKind::Identifier(id), ttype)
    }
    TypeExpressionKind::IdentifierWithParams { identifier, params } => {
      let finalized: Vec<TypeExpression<Typed>> = params
        .into_iter()
        .map(|p| finalize_type_expr(p, scope))
        .collect::<CompilerResult<_>>()?;
      let param_types = finalized.iter().map(|te| te.meta.ttype.clone()).collect();
      let ttype = Type::Builtin {
        id: identifier.clone(),
        params: param_types,
        linear: false,
      };
      (
        TypeExpressionKind::IdentifierWithParams {
          identifier,
          params: finalized,
        },
        ttype,
      )
    }
    TypeExpressionKind::Slice(inner) => {
      let inner = finalize_type_expr(*inner, scope)?;
      let ttype = Type::Slice(Box::new(inner.meta.ttype.clone()));
      (TypeExpressionKind::Slice(Box::new(inner)), ttype)
    }
    TypeExpressionKind::Tuple(types) => {
      let finalized: Vec<TypeExpression<Typed>> = types
        .into_iter()
        .map(|t| finalize_type_expr(t, scope))
        .collect::<CompilerResult<_>>()?;
      let inner_types = finalized.iter().map(|te| te.meta.ttype.clone()).collect();
      (
        TypeExpressionKind::Tuple(finalized),
        Type::Tuple(inner_types),
      )
    }
    TypeExpressionKind::Transform {
      input_type,
      output_type,
    } => {
      let input = finalize_type_expr(*input_type, scope)?;
      let output = finalize_type_expr(*output_type, scope)?;
      let ttype = Type::Arrow(
        Box::new(input.meta.ttype.clone()),
        Box::new(output.meta.ttype.clone()),
      );
      (
        TypeExpressionKind::Transform {
          input_type: Box::new(input),
          output_type: Box::new(output),
        },
        ttype,
      )
    }
  };
  Ok(TypeExpression {
    kind,
    meta: TypeMetadata {
      span,
      ttype,
      scope: Rc::clone(scope),
    },
  })
}

fn finalize_expr(
  expr: Expression<Inferred>,
  subst: &HashMap<usize, TypeWithUnknow>,
) -> CompilerResult<Expression<Typed>> {
  let meta = resolve_meta(expr.meta, subst)?;
  let kind = match expr.kind {
    ExpressionKind::LetBlock {
      strict,
      assigments,
      value,
    } => {
      let assigments = assigments
        .into_iter()
        .map(|(pat, e)| Ok((finalize_pattern(pat), finalize_expr(e, subst)?)))
        .collect::<CompilerResult<_>>()?;
      let value = finalize_expr(*value, subst)?;
      ExpressionKind::LetBlock {
        strict,
        assigments,
        value: Box::new(value),
      }
    }
    ExpressionKind::IfElseBlock {
      test,
      consequent,
      alternative,
    } => ExpressionKind::IfElseBlock {
      test: Box::new(finalize_expr(*test, subst)?),
      consequent: Box::new(finalize_expr(*consequent, subst)?),
      alternative: Box::new(finalize_expr(*alternative, subst)?),
    },
    ExpressionKind::Transform { callee, param } => ExpressionKind::Transform {
      callee: Box::new(finalize_expr(*callee, subst)?),
      param: Box::new(finalize_expr(*param, subst)?),
    },
    ExpressionKind::Lambda { params, body } => ExpressionKind::Lambda {
      params,
      body: Box::new(finalize_expr(*body, subst)?),
    },
    ExpressionKind::Tuple(exprs) => ExpressionKind::Tuple(
      exprs
        .into_iter()
        .map(|e| finalize_expr(e, subst))
        .collect::<CompilerResult<_>>()?,
    ),
    ExpressionKind::Slice(slice_expr) => ExpressionKind::Slice(match slice_expr {
      SliceExpression::Items(items) => SliceExpression::Items(
        items
          .into_iter()
          .map(|e| finalize_expr(e, subst))
          .collect::<CompilerResult<_>>()?,
      ),
      SliceExpression::Range(start, end) => SliceExpression::Range(
        Box::new(finalize_expr(*start, subst)?),
        Box::new(finalize_expr(*end, subst)?),
      ),
    }),
    ExpressionKind::Identifier(s) => ExpressionKind::Identifier(s),
    ExpressionKind::IntegerLiteral { number, suffix } => {
      ExpressionKind::IntegerLiteral { number, suffix }
    }
    ExpressionKind::FloatLiteral { number, suffix } => {
      ExpressionKind::FloatLiteral { number, suffix }
    }
  };
  Ok(Expression { kind, meta })
}

pub fn finalize_decl(
  decl: Declaration<Inferred>,
  subst: &HashMap<usize, TypeWithUnknow>,
) -> CompilerResult<Declaration<Typed>> {
  let scope = Rc::clone(&decl.meta.scope);
  let meta = resolve_meta(decl.meta, subst)?;
  match decl.kind {
    DeclarationKind::Function {
      identifier,
      type_annotation,
      params,
      body,
    } => {
      let type_annotation = finalize_type_expr(type_annotation, &scope)?;
      let body = finalize_expr(body, subst)?;
      Ok(Declaration {
        kind: DeclarationKind::Function {
          identifier,
          type_annotation,
          params,
          body,
        },
        meta,
      })
    }
    DeclarationKind::Type { .. } => unreachable!(),
  }
}
