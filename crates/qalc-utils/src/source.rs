use ariadne;
use std::{fs, io};

#[derive(Debug)]
pub struct Source {
  name: String,
  content: String,
}

impl Source {
  pub fn memory(content: impl Into<String>) -> Self {
    Source {
      name: "<memory>".to_string(),
      content: content.into(),
    }
  }

  pub fn file(path: impl Into<String>) -> io::Result<Self> {
    let source_file = path.into();
    let content = fs::read_to_string(&source_file)? + " ";

    Ok(Source {
      name: source_file,
      content,
    })
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn content(&self) -> &str {
    &self.content
  }

  pub fn to_ariadne_source(&self) -> (&str, ariadne::Source) {
    (&self.name, ariadne::Source::from(self.content.clone()))
  }
}
