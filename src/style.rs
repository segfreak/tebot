pub trait Style {
  fn err() -> &'static str;
  fn ok() -> &'static str;
  fn bullet() -> &'static str;
  fn info() -> &'static str;
  fn arrow() -> &'static str;
}

pub struct DefaultStyle;

impl Style for DefaultStyle {
  fn err() -> &'static str {
    "✕"
  }
  fn ok() -> &'static str {
    "✓"
  }
  fn bullet() -> &'static str {
    "⇛"
  }
  fn info() -> &'static str {
    "⇒"
  }
  fn arrow() -> &'static str {
    "⨠"
  }
}
