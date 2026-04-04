use std::{env::args, rc::Rc};

use qalc_parser::parse_program;
use qalc_utils::Source;

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
  println!("AST: {:#?}", concrete_ast);
}
