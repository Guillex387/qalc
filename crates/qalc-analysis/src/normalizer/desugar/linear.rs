use qalc_ast::concrete_ast::{
  Expression as CExpr, ExpressionKind as CExprKind, Pattern, PatternKind,
};
use qalc_ast::normalized_ast::{Expression as NExpr, ExpressionKind as NExprKind};
use qalc_utils::{CompilerResult, Diagnostic, Span};

use crate::normalizer::desugar::{transform_assigment_to_let, transform_expr_to_letstrict};
use crate::normalizer::expr::transform_expression;

fn extract_call_context(expr: &NExpr) -> CompilerResult<Vec<Pattern>> {
  if let NExprKind::Transform { callee: _, param } = expr.kind.clone() {
    if let NExprKind::Tuple(identifiers) = param.kind {
      identifiers
        .into_iter()
        .map(|expr| {
          if let NExprKind::Identifier(ident) = expr.kind {
            Ok(Pattern {
              kind: PatternKind::Variable(ident),
              span: expr.span.clone(),
            })
          } else {
            Err(Diagnostic::error(
              "Expected identifier",
              "The context of a linear call only allow identifiers",
              expr.span,
            ))
          }
        })
        .collect::<CompilerResult<Vec<Pattern>>>()
    } else if let NExprKind::Identifier(ident) = param.kind {
      Ok(vec![Pattern {
        kind: PatternKind::Variable(ident),
        span: param.span,
      }])
    } else {
      Err(Diagnostic::error(
        "Expected tuple or identifier",
        "The context of a linear call must be an identifier or a tuple of identifiers",
        param.span,
      ))
    }
  } else {
    Err(Diagnostic::error(
      "Expected call expression",
      "Linear assigments only allows call expressions",
      expr.span.clone(),
    ))
  }
}

fn transform_monadic_assigment_to_let(
  identifier: Pattern,
  value: Box<CExpr>,
  span: Span,
  wrapped_expr: NExpr,
) -> CompilerResult<NExpr> {
  let value = transform_expression(*value)?;
  let mut context_params = extract_call_context(&value)?;
  let identifier_span = identifier.span.clone();
  context_params.push(identifier);

  Ok(NExpr {
    kind: NExprKind::LetBlock {
      strict: false,
      assigments: vec![(
        Pattern {
          kind: PatternKind::Tuple(context_params),
          span: identifier_span,
        },
        value,
      )],
      value: Box::new(wrapped_expr),
    },
    span,
  })
}

pub fn transform_linear_block(
  sequence: Vec<CExpr>,
  value: Option<Box<CExpr>>,
) -> CompilerResult<NExprKind> {
  let last_span = sequence.last().unwrap().span.clone();
  let void_tuple = NExpr {
    kind: NExprKind::Tuple(Vec::new()),
    span: last_span,
  };
  let return_value = value.map_or(Ok(void_tuple), |expr| transform_expression(*expr))?;

  Ok(
    sequence
      .into_iter()
      .try_rfold(return_value, |result, sentence| match sentence.kind {
        CExprKind::Assigment {
          identifier,
          value,
          monadic,
        } => {
          if monadic {
            transform_monadic_assigment_to_let(identifier, value, sentence.span, result)
          } else {
            transform_assigment_to_let(identifier, value, sentence.span, result)
          }
        }
        _ => Ok(transform_expr_to_letstrict(
          transform_expression(sentence)?,
          result,
        )), // TODO: adds a warning to non-call expression (no execution)
      })?
      .kind,
  )
}
