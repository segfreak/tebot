use std::sync::{Arc, Weak};

use tokio::sync::Mutex;

use crate::bot::context::Context;

pub trait Style: Send + Sync {
  fn s_err() -> &'static str;
  fn s_ok() -> &'static str;
  fn s_bullet() -> &'static str;
  fn s_info() -> &'static str;
  fn s_arrow() -> &'static str;
}

pub trait DynStyle: Send + Sync {
  fn err(&self) -> &'static str;
  fn ok(&self) -> &'static str;
  fn bullet(&self) -> &'static str;
  fn info(&self) -> &'static str;
  fn arrow(&self) -> &'static str;
}

pub async fn get_style(ctx: Weak<Mutex<Context>>) -> Arc<dyn DynStyle> {
  match ctx.upgrade() {
    Some(ctx) => {
      let ctx_guard = ctx.lock().await;
      return ctx_guard.style.clone();
    }
    None => return Arc::new(DefaultStyle),
  };
}

pub struct DefaultStyle;

impl Style for DefaultStyle {
  fn s_err() -> &'static str {
    "✕"
  }
  fn s_ok() -> &'static str {
    "✓"
  }
  fn s_bullet() -> &'static str {
    "⇛"
  }
  fn s_info() -> &'static str {
    "⇒"
  }
  fn s_arrow() -> &'static str {
    "⨠"
  }
}

impl DynStyle for DefaultStyle {
  fn err(&self) -> &'static str {
    Self::s_err()
  }
  fn ok(&self) -> &'static str {
    Self::s_ok()
  }
  fn bullet(&self) -> &'static str {
    Self::s_bullet()
  }
  fn info(&self) -> &'static str {
    Self::s_info()
  }
  fn arrow(&self) -> &'static str {
    Self::s_arrow()
  }
}
