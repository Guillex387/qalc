use std::{env::args, rc::Rc};

use qalc_analysis::{HMInferrer, Normalizer};
use qalc_ast::base_ast::{
  Declaration, DeclarationKind, Expression, ExpressionKind, Pattern, PatternKind, SliceExpression,
};
use qalc_ast::typed_ast::{Type, TypedAST, Typed};
use qalc_builtins::standard_builtins;
use qalc_parser::parse_program;
use qalc_utils::{CompilerStep, Diagnostic, Source};
use serde_json::{Value, json};

fn main() {
  let filename = args().nth(1).expect("1 argument required");
  let source = Rc::new(Source::file(filename).expect("Error reading the source file"));

  let concrete_ast = match parse_program(&source) {
    Ok(ast) => ast,
    Err(error) => {
      eprintln!("{}", error);
      return;
    }
  };

  let mut diagnostics: Vec<Diagnostic> = Vec::new();

  let normalized_ast = match Normalizer::default().transform(concrete_ast, &mut diagnostics) {
    Ok(ast) => ast,
    Err(error) => {
      eprintln!("{}", error);
      return;
    }
  };

  let inferrer = HMInferrer::new(standard_builtins());
  let typed_ast = match inferrer.transform(normalized_ast, &mut diagnostics) {
    Ok(ast) => ast,
    Err(error) => {
      eprintln!("{}", error);
      return;
    }
  };

  let json = ast_to_json(&typed_ast);
  println!("{}", serde_json::to_string_pretty(&json).unwrap());
}

fn fmt_type(t: &Type) -> String {
  match t {
    Type::Builtin { id, params, .. } if params.is_empty() => id.clone(),
    Type::Builtin { id, params, .. } => format!(
      "{}<{}>",
      id,
      params.iter().map(fmt_type).collect::<Vec<_>>().join(", ")
    ),
    Type::Arrow(a, b) => format!("{} -> {}", fmt_type_parens(a), fmt_type(b)),
    Type::Tuple(ts) => format!("({})", ts.iter().map(fmt_type).collect::<Vec<_>>().join(", ")),
    Type::Slice(t) => format!("[{}]", fmt_type(t)),
  }
}

fn fmt_type_parens(t: &Type) -> String {
  match t {
    Type::Arrow(_, _) => format!("({})", fmt_type(t)),
    _ => fmt_type(t),
  }
}

fn pattern_to_json(p: &Pattern<Typed>) -> Value {
  match &p.kind {
    PatternKind::Variable(name) => json!(name),
    PatternKind::Wildcard => json!("_"),
    PatternKind::Tuple(pats) => {
      Value::Array(pats.iter().map(pattern_to_json).collect())
    }
  }
}

fn expr_to_json(expr: &Expression<Typed>) -> Value {
  let t = fmt_type(&expr.meta.ttype);
  match &expr.kind {
    ExpressionKind::Identifier(name) => json!({ "ident": name, "type": t }),
    ExpressionKind::IntegerLiteral { number, .. } => json!({ "int": number, "type": t }),
    ExpressionKind::FloatLiteral { number, .. } => json!({ "float": number, "type": t }),
    ExpressionKind::Tuple(exprs) => json!({
      "tuple": exprs.iter().map(expr_to_json).collect::<Vec<_>>(),
      "type": t,
    }),
    ExpressionKind::Transform { callee, param } => json!({
      "app": {
        "fn": expr_to_json(callee),
        "arg": expr_to_json(param),
      },
      "type": t,
    }),
    ExpressionKind::Lambda { params, body } => json!({
      "lambda": {
        "params": params,
        "body": expr_to_json(body),
      },
      "type": t,
    }),
    ExpressionKind::LetBlock { strict, assigments, value } => {
      let bindings: Vec<Value> = assigments
        .iter()
        .map(|(pat, expr)| json!({ "pat": pattern_to_json(pat), "val": expr_to_json(expr) }))
        .collect();
      json!({
        "let": {
          "strict": strict,
          "bindings": bindings,
          "in": expr_to_json(value),
        },
        "type": t,
      })
    }
    ExpressionKind::IfElseBlock { test, consequent, alternative } => json!({
      "if": {
        "test": expr_to_json(test),
        "then": expr_to_json(consequent),
        "else": expr_to_json(alternative),
      },
      "type": t,
    }),
    ExpressionKind::Slice(slice) => match slice {
      SliceExpression::Items(items) => json!({
        "slice": items.iter().map(expr_to_json).collect::<Vec<_>>(),
        "type": t,
      }),
      SliceExpression::Range(start, end) => json!({
        "range": { "start": expr_to_json(start), "end": expr_to_json(end) },
        "type": t,
      }),
    },
  }
}

fn decl_to_json(decl: &Declaration<Typed>) -> Value {
  match &decl.kind {
    DeclarationKind::Function { identifier, params, body, .. } => json!({
      "fn": identifier,
      "params": params,
      "type": fmt_type(&decl.meta.ttype),
      "body": expr_to_json(body),
    }),
    DeclarationKind::Type { .. } => json!(null),
  }
}

fn ast_to_json(ast: &TypedAST) -> Value {
  json!({
    "declarations": ast.declarations.iter().map(decl_to_json).collect::<Vec<_>>(),
  })
}
