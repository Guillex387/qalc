use qalc_ast::{
  ASTTransform,
  concrete_ast::{
    self, BinaryOperator, BinaryOperatorKind, ConcreteAST, Declaration as CDecl,
    DeclarationKind as CDeclKind, Expression as CExpr, ExpressionKind as CExprKind, Pattern,
    TypeExpression,
  },
  normalized_ast::{
    self, Declaration as NDecl, DeclarationKind as NDeclKind, Expression as NExpr,
    ExpressionKind as NExprKind, NormalizedAST,
  },
};
use qalc_utils::{CompilerResult, Diagnostic, Span};

fn transform_let_expression(assigments: Vec<CExpr>, value: Box<CExpr>) -> NExprKind {
  let assigments = assigments
    .into_iter()
    .map(|assigment| {
      if let CExprKind::Assigment {
        identifier, value, ..
      } = assigment.kind
      {
        let value = transform_expression(*value);
        (identifier, value)
      } else {
        unreachable!();
      }
    })
    .collect();
  let value = Box::new(transform_expression(*value));

  NExprKind::LetBlock { assigments, value }
}

fn transform_match(
  value: Box<CExpr>,
  cases: Vec<(CExpr, CExpr)>,
  default: Option<Box<CExpr>>,
  span: Span,
) -> NExprKind {
  let value = Box::new(transform_expression(*value));
  let mut if_block = default
    .map(|expr| Box::new(transform_expression(*expr)))
    .unwrap_or(Box::new(NExpr {
      kind: NExprKind::Tuple(Vec::new()),
      span,
    }));

  for (check_value, return_value) in cases {
    let check_value = transform_expression(check_value);
    let return_value = transform_expression(return_value);

    let case_span = check_value.span.join(&return_value.span);
    let eq_call = NExpr {
      span: case_span.clone(),
      kind: NExprKind::Call {
        callee: Box::new(NExpr {
          kind: NExprKind::Identifier("==".to_string()),
          span: case_span.clone(),
        }),
        params: vec![*value.clone(), check_value],
      },
    };

    if_block = Box::new(NExpr {
      span: case_span,
      kind: NExprKind::IfElseBlock {
        test: Box::new(eq_call),
        consequent: Box::new(return_value),
        alternative: if_block,
      },
    })
  }

  if_block.kind
}

fn operator_to_identifier(operator: BinaryOperator) -> NExpr {
  let ident = match &operator.kind {
    BinaryOperatorKind::Apply => "$",
    BinaryOperatorKind::Pipe => "|>",
    BinaryOperatorKind::LogicOr => "||",
    BinaryOperatorKind::LogicAnd => "&&",
    BinaryOperatorKind::Or => "|",
    BinaryOperatorKind::And => "&",
    BinaryOperatorKind::Xor => "^",
    BinaryOperatorKind::Equal => "==",
    BinaryOperatorKind::NotEqual => "!=",
    BinaryOperatorKind::LessEqual => "<=",
    BinaryOperatorKind::GreaterEqual => ">=",
    BinaryOperatorKind::Less => "<",
    BinaryOperatorKind::Greater => ">",
    BinaryOperatorKind::Add => "+",
    BinaryOperatorKind::Subtract => "-",
    BinaryOperatorKind::Multiply => "*",
    BinaryOperatorKind::Divide => "/",
    BinaryOperatorKind::Module => "%",
    BinaryOperatorKind::Raised => "**",
  }
  .to_string();

  NExpr {
    kind: NExprKind::Identifier(ident),
    span: operator.span,
  }
}

fn transform_slice(slice: concrete_ast::SliceExpression) -> NExprKind {
  let slice_expr = match slice {
    concrete_ast::SliceExpression::Items(items) => {
      normalized_ast::SliceExpression::Items(items.into_iter().map(transform_expression).collect())
    }
    concrete_ast::SliceExpression::Range(start, end) => {
      let start = Box::new(transform_expression(*start));
      let end = Box::new(transform_expression(*end));

      normalized_ast::SliceExpression::Range(start, end)
    }
  };

  NExprKind::Slice(slice_expr)
}

fn transform_expression(expression: CExpr) -> NExpr {
  let span = expression.span;
  let kind = match expression.kind {
    CExprKind::LetBlock { assigments, value } => transform_let_expression(assigments, value),
    CExprKind::IfElseBlock {
      test,
      consequent,
      alternative,
    } => {
      let test = Box::new(transform_expression(*test));
      let consequent = Box::new(transform_expression(*consequent));
      let alternative = Box::new(transform_expression(*alternative));

      NExprKind::IfElseBlock {
        test,
        consequent,
        alternative,
      }
    }
    CExprKind::MatchBlock {
      value,
      cases,
      default,
    } => transform_match(value, cases, default, span.clone()),
    CExprKind::PerformBlock { sequence, value } => todo!(),
    CExprKind::LinearBlock { sequence, value } => todo!(),
    CExprKind::Binary {
      left,
      operator,
      right,
    } => NExprKind::Call {
      callee: Box::new(operator_to_identifier(operator)),
      params: vec![transform_expression(*left), transform_expression(*right)],
    },
    CExprKind::PartialLeftBinary { operator, right } => todo!(),
    CExprKind::PartialRightBinary { left, operator } => todo!(),
    CExprKind::Call { callee, params } => {
      let callee = Box::new(transform_expression(*callee));
      let params = params.into_iter().map(transform_expression).collect();

      NExprKind::Call { callee, params }
    }
    CExprKind::Lambda { params, body } => {
      let (params, body) = transform_params_pattern_to_let(params, *body);

      NExprKind::Lambda {
        params,
        body: Box::new(body),
      }
    }
    CExprKind::Tuple(tuple) => {
      NExprKind::Tuple(tuple.into_iter().map(transform_expression).collect())
    }
    CExprKind::Slice(slice) => transform_slice(slice),
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
