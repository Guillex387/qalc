use std::env::args;

use qalc_parser::parse_program;

fn main() {
  let filename = args().nth(1).expect("1 argument required");
  let concrete_ast = match parse_program(&filename) {
    Ok(ast) => ast,
    Err(error) => {
      eprintln!("{}", error);
      return;
    }
  };
  println!("AST: {:#?}", concrete_ast);
}
