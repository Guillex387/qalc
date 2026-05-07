use crate::Spanned;
use crate::base_ast::Declaration;

#[derive(Debug, Clone)]
pub struct NormalizedAST {
  pub declarations: Vec<Declaration<Spanned>>,
}
