use std::sync::{Arc, Weak};

use tokio::sync::Mutex;

use crate::context::Context;

pub trait Style: Send + Sync {
  fn err() -> &'static str;
  fn ok() -> &'static str;
  fn bullet() -> &'static str;
  fn info() -> &'static str;
  fn arrow() -> &'static str;
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
    None => return Arc::new(DefaultDynStyle),
  };
}

pub struct DefaultStyle;
pub struct DefaultDynStyle;

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

impl DynStyle for DefaultDynStyle {
  fn err(&self) -> &'static str {
    DefaultStyle::err()
  }
  fn ok(&self) -> &'static str {
    DefaultStyle::ok()
  }
  fn bullet(&self) -> &'static str {
    DefaultStyle::bullet()
  }
  fn info(&self) -> &'static str {
    DefaultStyle::info()
  }
  fn arrow(&self) -> &'static str {
    DefaultStyle::arrow()
  }
}
