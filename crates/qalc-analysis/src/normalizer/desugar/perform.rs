use qalc_ast::concrete_ast::{
  Expression as CExpr, ExpressionKind as CExprKind, Pattern, PatternKind,
};
use qalc_ast::normalized_ast::{Expression as NExpr, ExpressionKind as NExprKind};
use qalc_utils::Span;

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
  identifier: Pattern,
  value: Box<CExpr>,
  span: Span,
  wrapped_expr: NExpr,
) -> NExpr {
  let monad_map_call = NExpr {
    kind: NExprKind::Transform {
      callee: Box::new(NExpr {
        kind: NExprKind::Identifier("monadMap".to_owned()),
        span: span.clone(),
      }),
      param: Box::new(transform_expression(*value)),
    },
    span: span.clone(),
  };
  let pattern_span = identifier.span.clone();
  let map_callback = NExpr {
    kind: NExprKind::Lambda {
      params: vec!["a".to_owned()],
      body: Box::new(NExpr {
        span: pattern_span.clone(),
        kind: NExprKind::LetBlock {
          strict: false,
          assigments: vec![(
            identifier,
            NExpr {
              kind: NExprKind::Identifier("a".to_owned()),
              span: pattern_span,
            },
          )],
          value: Box::new(wrapped_expr),
        },
      }),
    },
    span: span.clone(),
  };

  NExpr {
    kind: NExprKind::Transform {
      callee: Box::new(monad_map_call),
      param: Box::new(map_callback),
    },
    span,
  }
}

// Example (source):
// perform {
//   ...
//   pureResult = 3 + 4,
//   ...
// }
// Example (normalized):
// ... let pureResult = `+` 3 4 in ...
fn transform_assigment_to_let(
  identifier: Pattern,
  value: Box<CExpr>,
  span: Span,
  wrapped_expr: NExpr,
) -> NExpr {
  NExpr {
    kind: NExprKind::LetBlock {
      strict: false,
      assigments: vec![(identifier, transform_expression(*value))],
      value: Box::new(wrapped_expr),
    },
    span,
  }
}

// Example (source):
// perform {
//   ...
//   print someThing,
//   ...
// }
// Example (normalized):
// ... letstrict _ = print someThing in ...
fn transform_expr_to_letstrict(expr: NExpr, wrapped_expr: NExpr) -> NExpr {
  NExpr {
    span: expr.span.clone(),
    kind: NExprKind::LetBlock {
      strict: true,
      assigments: vec![(
        Pattern {
          kind: PatternKind::Wildcard,
          span: expr.span.clone(),
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
fn transform_expr_to_monad_pure(expr: NExpr) -> NExpr {
  NExpr {
    span: expr.span.clone(),
    kind: NExprKind::Transform {
      callee: Box::new(NExpr {
        kind: NExprKind::Identifier("monadPure".to_owned()),
        span: expr.span.clone(),
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
pub fn transform_perform_block(sequence: Vec<CExpr>, value: Option<Box<CExpr>>) -> NExprKind {
  let last_span = sequence.last().unwrap().span.clone();
  let void_tuple = NExpr {
    kind: NExprKind::Tuple(Vec::new()),
    span: last_span,
  };
  let return_value =
    transform_expr_to_monad_pure(value.map_or(void_tuple, |expr| transform_expression(*expr)));

  sequence
    .into_iter()
    .rfold(return_value, |result, sentence| match sentence.kind {
      CExprKind::Assigment {
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
      _ => transform_expr_to_letstrict(transform_expression(sentence), result), // TODO: adds a warning to non-call expression (no execution)
    })
    .kind
}
