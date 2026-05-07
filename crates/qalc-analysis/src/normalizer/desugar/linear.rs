use qalc_ast::Spanned;
use qalc_ast::base_ast::{Expression, ExpressionKind, Pattern, PatternKind};
use qalc_ast::concrete_ast::{ConcreteExpression, ConcreteExpressionKind};
use qalc_utils::{CompilerResult, Diagnostic, Span};

use crate::normalizer::desugar::{transform_assigment_to_let, transform_expr_to_letstrict};
use crate::normalizer::expr::transform_expression;

fn extract_call_context(expr: &Expression<Spanned>) -> CompilerResult<Vec<Pattern<Spanned>>> {
  if let ExpressionKind::Transform { callee: _, param } = expr.kind.clone() {
    if let ExpressionKind::Tuple(identifiers) = param.kind {
      identifiers
        .into_iter()
        .map(|expr| {
          if let ExpressionKind::Identifier(ident) = expr.kind {
            Ok(Pattern {
              kind: PatternKind::Variable(ident),
              meta: expr.meta.clone(),
            })
          } else {
            Err(Diagnostic::error(
              "Expected identifier",
              "The context of a linear call only allow identifiers",
              expr.meta,
            ))
          }
        })
        .collect::<CompilerResult<_>>()
    } else if let ExpressionKind::Identifier(ident) = param.kind {
      Ok(vec![Pattern {
        kind: PatternKind::Variable(ident),
        meta: param.meta,
      }])
    } else {
      Err(Diagnostic::error(
        "Expected tuple or identifier",
        "The context of a linear call must be an identifier or a tuple of identifiers",
        param.meta,
      ))
    }
  } else {
    Err(Diagnostic::error(
      "Expected call expression",
      "Linear assigments only allows call expressions",
      expr.meta.clone(),
    ))
  }
}

fn transform_monadic_assigment_to_let(
  identifier: Pattern<Spanned>,
  value: Box<ConcreteExpression>,
  span: Span,
  wrapped_expr: Expression<Spanned>,
) -> CompilerResult<Expression<Spanned>> {
  let value = transform_expression(*value)?;
  let mut context_params = extract_call_context(&value)?;
  let identifier_span = identifier.meta.clone();
  context_params.push(identifier);

  Ok(Expression {
    kind: ExpressionKind::LetBlock {
      strict: false,
      assigments: vec![(
        Pattern {
          kind: PatternKind::Tuple(context_params),
          meta: identifier_span,
        },
        value,
      )],
      value: Box::new(wrapped_expr),
    },
    meta: span,
  })
}

pub fn transform_linear_block(
  sequence: Vec<ConcreteExpression>,
  value: Option<Box<ConcreteExpression>>,
) -> CompilerResult<ExpressionKind<Spanned>> {
  let last_span = sequence.last().unwrap().span.clone();
  let void_tuple = Expression {
    kind: ExpressionKind::Tuple(Vec::new()),
    meta: last_span,
  };
  let return_value = value.map_or(Ok(void_tuple), |expr| transform_expression(*expr))?;

  Ok(
    sequence
      .into_iter()
      .try_rfold(return_value, |result, sentence| match sentence.kind {
        ConcreteExpressionKind::Assigment {
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
