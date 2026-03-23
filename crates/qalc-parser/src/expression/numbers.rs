use pest::iterators::Pair;
use qalc_ast::{ NumberSuffix, concrete_ast::ExpressionKind };

use crate::Rule;

fn suffix_for_rule(rule: Rule) -> (NumberSuffix, Option<&'static str>) {
  match rule {
    Rule::u8_integer => (NumberSuffix::U8, Some("u8")),
    Rule::u16_integer => (NumberSuffix::U16, Some("u16")),
    Rule::u32_integer => (NumberSuffix::U32, Some("u32")),
    Rule::u64_integer => (NumberSuffix::U64, Some("u64")),
    Rule::i8_integer => (NumberSuffix::I8, Some("i8")),
    Rule::i16_integer => (NumberSuffix::I16, Some("i16")),
    Rule::i32_integer => (NumberSuffix::I32, Some("i32")),
    Rule::i64_integer => (NumberSuffix::I64, Some("i64")),
    Rule::unknow_integer => (NumberSuffix::Unknow, None),
    rule => unreachable!("Expected integer literal, found {:?}", rule),
  }
}

fn to_decimal_string(text: &str) -> String {
  if let Some(hex) = text.strip_prefix("0x") {
    match u128::from_str_radix(hex, 16) {
      Ok(value) => value.to_string(),
      Err(_) => text.to_string(),
    }
  } else if let Some(bin) = text.strip_prefix("0b") {
    match u128::from_str_radix(bin, 2) {
      Ok(value) => value.to_string(),
      Err(_) => text.to_string(),
    }
  } else {
    text.to_string()
  }
}

pub fn parse_number_literal(primary: Pair<Rule>) -> ExpressionKind {
  let (suffix, suffix_text) = suffix_for_rule(primary.as_rule());
  let raw_text = primary.as_str();
  let number_text = match suffix_text {
    Some(suffix_text) => raw_text.strip_suffix(suffix_text).unwrap_or(raw_text),
    None => raw_text,
  };

  ExpressionKind::NumberLiteral {
    number: to_decimal_string(number_text),
    suffix,
  }
}
