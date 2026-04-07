use qalc_ast::{
  ASTTransform,
  concrete_ast::{
    ConcreteAST, Declaration as CDecl, DeclarationKind as CDeclKind, Expression as CExpr,
    ExpressionKind as CExprKind, Pattern, TypeExpression,
  },
  normalized_ast::{
    Declaration as NDecl, DeclarationKind as NDeclKind, Expression as NExpr,
    ExpressionKind as NExprKind, NormalizedAST,
  },
};
use qalc_utils::{CompilerResult, Diagnostic};

fn transform_expression(expression: CExpr) -> NExpr {
  let span = expression.span;
  let kind = match expression.kind {
    CExprKind::LetBlock { assigments, value } => todo!(),
    CExprKind::IfElseBlock {
      test,
      consequent,
      alternative,
    } => todo!(),
    CExprKind::MatchBlock {
      value,
      cases,
      default,
    } => todo!(),
    CExprKind::PerformBlock { sequence, value } => todo!(),
    CExprKind::LinearBlock { sequence, value } => todo!(),
    CExprKind::Binary {
      left,
      operator,
      right,
    } => todo!(),
    CExprKind::PartialLeftBinary { operator, right } => todo!(),
    CExprKind::PartialRightBinary { left, operator } => todo!(),
    CExprKind::Call { callee, params } => todo!(),
    CExprKind::Lambda { params, body } => todo!(),
    CExprKind::Tuple(tuple) => {
      NExprKind::Tuple(tuple.into_iter().map(transform_expression).collect())
    }
    CExprKind::Slice(slice) => todo!(),
    CExprKind::Identifier(identifier) => NExprKind::Identifier(identifier),
    CExprKind::IntegerLiteral { number, suffix } => NExprKind::IntegerLiteral { number, suffix },
    CExprKind::FloatLiteral { number, suffix } => NExprKind::FloatLiteral { number, suffix },
    CExprKind::Assigment { .. } => unreachable!(),
  };

  NExpr { kind, span }
}

fn transform_params_pattern_to_let(params: Vec<Pattern>, body: CExpr) -> (Vec<String>, NExpr) {
  let norm_params: Vec<String> = params
    .iter()
    .enumerate()
    .map(|(index, _)| format!("p{}", index).to_string())
    .collect();
  let span = params
    .first()
    .unwrap()
    .span
    .join(&params.last().unwrap().span);

  let new_body = NExpr {
    kind: NExprKind::LetBlock {
      assigments: params
        .into_iter()
        .zip(norm_params.clone())
        .map(|(pattern, name)| {
          (
            pattern.clone(),
            NExpr {
              kind: NExprKind::Identifier(name),
              span: pattern.span,
            },
          )
        })
        .collect(),
      value: Box::new(transform_expression(body)),
    },
    span,
  };

  (norm_params, new_body)
}

fn transform_function_decl(
  identifier: String,
  type_annotation: TypeExpression,
  params: Vec<Pattern>,
  body: CExpr,
) -> NDeclKind {
  if params.is_empty() {
    return NDeclKind::Function {
      identifier,
      type_annotation,
      params: Vec::new(),
      body: transform_expression(body),
    };
  }

  let (params, body) = transform_params_pattern_to_let(params, body);

  NDeclKind::Function {
    identifier,
    type_annotation,
    params,
    body,
  }
}

fn transform_declaration(declaration: CDecl) -> NDecl {
  let span = declaration.span;
  let kind = match declaration.kind {
    CDeclKind::Type {
      identifier,
      attributes,
      linear,
    } => NDeclKind::Type {
      identifier,
      attributes,
      linear,
    },
    CDeclKind::Function {
      identifier,
      type_annotation,
      params,
      body,
    } => transform_function_decl(identifier, type_annotation, params, body),
  };

  NDecl { kind, span }
}

#[derive(Default)]
pub struct Normalizer;

impl ASTTransform for Normalizer {
  type Input = ConcreteAST;
  type Output = NormalizedAST;

  fn transform(&self, input: Self::Input, _: &mut Vec<Diagnostic>) -> CompilerResult<Self::Output> {
    Ok(NormalizedAST {
      declarations: input
        .declarations
        .into_iter()
        .map(transform_declaration)
        .collect(),
    })
  }
}
