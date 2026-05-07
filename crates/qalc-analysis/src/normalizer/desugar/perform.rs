use qalc_ast::Spanned;
use qalc_ast::base_ast::{Expression, ExpressionKind, Pattern, PatternKind};
use qalc_ast::concrete_ast::{ConcreteExpression, ConcreteExpressionKind};
use qalc_utils::{CompilerResult, Span};

use crate::normalizer::expr::transform_expression;

// Example (source):
// perform {
//   ...
//   result <- measure qbit
//   ...
// }
// Example (normalized):
// ... monadMap (measure qbit) (a) -> let result = a in ...
fn transform_monad_assigments_to_monadmap(
  identifier: Pattern<Spanned>,
  value: Box<ConcreteExpression>,
  span: Span,
  wrapped_expr: Expression<Spanned>,
) -> CompilerResult<Expression<Spanned>> {
  let monad_map_call = Expression {
    kind: ExpressionKind::Transform {
      callee: Box::new(Expression {
        kind: ExpressionKind::Identifier("monadMap".to_owned()),
        meta: span.clone(),
      }),
      param: Box::new(transform_expression(*value)?),
    },
    meta: span.clone(),
  };
  let pattern_span = identifier.meta.clone();
  let map_callback = Expression {
    kind: ExpressionKind::Lambda {
      params: vec!["a".to_owned()],
      body: Box::new(Expression {
        meta: pattern_span.clone(),
        kind: ExpressionKind::LetBlock {
          strict: false,
          assigments: vec![(
            identifier,
            Expression {
              kind: ExpressionKind::Identifier("a".to_owned()),
              meta: pattern_span,
            },
          )],
          value: Box::new(wrapped_expr),
        },
      }),
    },
    meta: span.clone(),
  };

  Ok(Expression {
    kind: ExpressionKind::Transform {
      callee: Box::new(monad_map_call),
      param: Box::new(map_callback),
    },
    meta: span,
  })
}

// Example (source):
// perform {
//   ...
//   pureResult = 3 + 4,
//   ...
// }
// Example (normalized):
// ... let pureResult = `+` 3 4 in ...
pub fn transform_assigment_to_let(
  identifier: Pattern<Spanned>,
  value: Box<ConcreteExpression>,
  span: Span,
  wrapped_expr: Expression<Spanned>,
) -> CompilerResult<Expression<Spanned>> {
  Ok(Expression {
    kind: ExpressionKind::LetBlock {
      strict: false,
      assigments: vec![(identifier, transform_expression(*value)?)],
      value: Box::new(wrapped_expr),
    },
    meta: span,
  })
}

// Example (source):
// perform {
//   ...
//   print someThing,
//   ...
// }
// Example (normalized):
// ... letstrict _ = print someThing in ...
pub fn transform_expr_to_letstrict(
  expr: Expression<Spanned>,
  wrapped_expr: Expression<Spanned>,
) -> Expression<Spanned> {
  Expression {
    meta: expr.meta.clone(),
    kind: ExpressionKind::LetBlock {
      strict: true,
      assigments: vec![(
        Pattern {
          kind: PatternKind::Wildcard,
          meta: expr.meta.clone(),
        },
        expr,
      )],
      value: Box::new(wrapped_expr),
    },
  }
}

// Example (source):
// perform {
//   ...
//   return 3 + 4
// }
// Example (normalized):
// ... monadPure (`+` 3 4)
fn transform_expr_to_monad_pure(expr: Expression<Spanned>) -> Expression<Spanned> {
  Expression {
    meta: expr.meta.clone(),
    kind: ExpressionKind::Transform {
      callee: Box::new(Expression {
        kind: ExpressionKind::Identifier("monadPure".to_owned()),
        meta: expr.meta.clone(),
      }),
      param: Box::new(expr),
    },
  }
}

// Example (source):
// perform {
//   result <- measure qbit,
//   otherThing = 3 + 4 * result,
//   print otherThing,
//   otherResult <- measure otherThing,
//   return otherResult
// }
// Example (normalized):
// flatMap
//  (measure qbit)
//  (result) ->
//    let
//      otherThing = `+` 3 (`*` 4 result)
//    in
//      letstrict
//        _ = print otherThing
//      in flatMap (measure otherThing) (otherResult) -> pure otherResult
pub fn transform_perform_block(
  sequence: Vec<ConcreteExpression>,
  value: Option<Box<ConcreteExpression>>,
) -> CompilerResult<ExpressionKind<Spanned>> {
  let last_span = sequence.last().unwrap().span.clone();
  let void_tuple = Expression {
    kind: ExpressionKind::Tuple(Vec::new()),
    meta: last_span,
  };
  let return_value = transform_expr_to_monad_pure(match value {
    Some(concrete_expr) => transform_expression(*concrete_expr)?,
    None => void_tuple,
  });

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
            transform_monad_assigments_to_monadmap(identifier, value, sentence.span, result)
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
