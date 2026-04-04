use qalc_ast::concrete_ast::{ExpressionKind, FloatSuffix, NumberSuffix};

use crate::tokenizer::{Rule, Token};

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

fn float_suffix_for_rule(rule: Rule) -> (FloatSuffix, Option<&'static str>) {
  match rule {
    Rule::f32_float => (FloatSuffix::F32, Some("f32")),
    Rule::f64_float => (FloatSuffix::F64, Some("f64")),
    Rule::unknow_float => (FloatSuffix::Unknow, None),
    rule => unreachable!("Expected float literal, found {:?}", rule),
  }
}

fn parse_integer_value(text: &str) -> i128 {
  let parsed = if let Some(hex) = text.strip_prefix("0x") {
    i128::from_str_radix(hex, 16)
  } else if let Some(bin) = text.strip_prefix("0b") {
    i128::from_str_radix(bin, 2)
  } else {
    text.parse::<i128>()
  };

  parsed.unwrap_or_else(|_| panic!("Invalid integer literal: {}", text))
}

fn normalize_float_text(text: &str) -> String {
  if text.starts_with('.') {
    format!("0{}", text)
  } else {
    text.to_string()
  }
}

fn parse_float_value(text: &str, suffix: FloatSuffix) -> f64 {
  let normalized = normalize_float_text(text);
  match suffix {
    FloatSuffix::F32 => normalized
      .parse::<f32>()
      .map(|value| value as f64)
      .unwrap_or_else(|_| panic!("Invalid f32 literal: {}", text)),
    FloatSuffix::F64 | FloatSuffix::Unknow => normalized
      .parse::<f64>()
      .unwrap_or_else(|_| panic!("Invalid f64 literal: {}", text)),
  }
}

pub fn parse_number_literal(token: Token) -> ExpressionKind {
  match token.rule() {
    Rule::unknow_integer
    | Rule::i8_integer
    | Rule::i16_integer
    | Rule::i32_integer
    | Rule::i64_integer
    | Rule::u8_integer
    | Rule::u16_integer
    | Rule::u32_integer
    | Rule::u64_integer => {
      let (suffix, suffix_text) = suffix_for_rule(token.rule());
      let raw_text = token.to_string();
      let number_text = match suffix_text {
        Some(suffix_text) => raw_text.strip_suffix(suffix_text).unwrap_or(&raw_text),
        None => &raw_text,
      };

      ExpressionKind::IntegerLiteral {
        number: parse_integer_value(number_text),
        suffix,
      }
    }
    Rule::unknow_float | Rule::f32_float | Rule::f64_float => {
      let (suffix, suffix_text) = float_suffix_for_rule(token.rule());
      let raw_text = token.to_string();
      let number_text = match suffix_text {
        Some(suffix_text) => raw_text.strip_suffix(suffix_text).unwrap_or(&raw_text),
        None => &raw_text,
      };

      ExpressionKind::FloatLiteral {
        number: parse_float_value(number_text, suffix.clone()),
        suffix,
      }
    }
    rule => unreachable!("Expected numeric literal, found {:?}", rule),
  }
}
