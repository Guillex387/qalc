use std::fmt;

use ariadne::{Color, Label, Report, ReportKind};

use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
  Note,
  Warning,
  Error,
}

impl DiagnosticKind {
  pub fn ariadne_color(&self) -> Color {
    match self {
      &Self::Error => Color::Red,
      &Self::Warning => Color::Yellow,
      &Self::Note => Color::Blue,
    }
  }
}

impl<'a> From<DiagnosticKind> for ReportKind<'a> {
  fn from(kind: DiagnosticKind) -> Self {
    match kind {
      DiagnosticKind::Note => ReportKind::Advice,
      DiagnosticKind::Warning => ReportKind::Warning,
      DiagnosticKind::Error => ReportKind::Error,
    }
  }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
  pub kind: DiagnosticKind,
  pub category: String,
  pub message: String,
  pub span: Span,
}

impl Diagnostic {
  pub fn note(category: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
    Self {
      kind: DiagnosticKind::Note,
      category: category.into(),
      message: message.into(),
      span,
    }
  }

  pub fn warning(category: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
    Self {
      kind: DiagnosticKind::Warning,
      category: category.into(),
      message: message.into(),
      span,
    }
  }

  pub fn error(category: impl Into<String>, message: impl Into<String>, span: Span) -> Self {
    Self {
      kind: DiagnosticKind::Error,
      category: category.into(),
      message: message.into(),
      span,
    }
  }

  fn render(&self) -> String {
    let mut output_buffer: Vec<u8> = Vec::new();
    let span = self.span.to_ariadne_span();
    let source = self.span.to_ariadne_source();

    Report::build(self.kind.into(), span.clone())
      .with_message(&self.category)
      .with_label(
        Label::new(span)
          .with_message(&self.message)
          .with_color(self.kind.ariadne_color()),
      )
      .finish()
      .write(source, &mut output_buffer)
      .unwrap();

    String::from_utf8_lossy(&output_buffer).to_string()
  }
}

impl std::error::Error for Diagnostic {}

impl fmt::Display for Diagnostic {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.render())
  }
}

pub type CompilerResult<T> = Result<T, Diagnostic>;
