use std::{env::args, rc::Rc};

use qalc_analysis::Normalizer;

use qalc_parser::parse_program;
use qalc_utils::{CompilerStep, Diagnostic, Source};

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
  println!("Concrete AST: {:#?}", concrete_ast);
  let normalizer = Normalizer::default();
  let mut diagnostics: Vec<Diagnostic> = Vec::new();
  let normalized_ast = match normalizer.transform(concrete_ast, &mut diagnostics) {
    Ok(ast) => ast,
    Err(error) => {
      eprintln!("{}", error);
      return;
    }
  };
  println!("Normalized AST: {:#?}", normalized_ast);
}
