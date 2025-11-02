use std::sync::{Arc, Weak};

use tokio::sync::Mutex;

use crate::bot::context::Context;

/// Defines a static text style used for visual elements such as
/// error marks, success symbols, and informational icons.
///
/// This trait is designed for compile-time style definitions where
/// all formatting symbols are constant. It provides an easy way to
/// customize the visual appearance of bot messages.
///
/// Implementations must be thread-safe (`Send + Sync`).
pub trait Style: Send + Sync {
  /// Returns the symbol used to indicate an error state.
  fn s_err() -> &'static str;

  /// Returns the symbol used to indicate a successful operation.
  fn s_ok() -> &'static str;

  /// Returns a bullet-like symbol, typically used for list items.
  fn s_bullet() -> &'static str;

  /// Returns an informational symbol, often used for hints or tips.
  fn s_info() -> &'static str;

  /// Returns an arrow-like symbol, used for directional or sequential flow.
  fn s_arrow() -> &'static str;
}

/// Represents a runtime-resolvable text style with the same set
/// of formatting symbols as [`Style`].
///
/// This trait allows using dynamic, heap-allocated style instances
/// (`Arc<dyn DynStyle>`) instead of compile-time constants, making
/// it suitable for customizable UI or per-user preferences.
pub trait DynStyle: Send + Sync {
  /// Returns the runtime symbol for an error state.
  fn err(&self) -> &'static str;

  /// Returns the runtime symbol for a successful operation.
  fn ok(&self) -> &'static str;

  /// Returns the runtime bullet symbol.
  fn bullet(&self) -> &'static str;

  /// Returns the runtime informational symbol.
  fn info(&self) -> &'static str;

  /// Returns the runtime arrow symbol.
  fn arrow(&self) -> &'static str;
}

/// Retrieves the active [`DynStyle`] instance from the given bot context.
///
/// If the provided [`Weak`] reference to [`Context`] is still valid,
/// this function locks the context and clones its current style handle.
/// Otherwise, it falls back to a default style implementation.
///
/// # Arguments
/// * `ctx` — A weak reference to the shared bot [`Context`].
///
/// # Returns
/// Returns an [`Arc<dyn DynStyle>`] representing the active or default style.
///
/// # Example
/// ```rust,ignore
/// let style = get_style(ctx.clone()).await;
/// println!("{}", style.info());
/// ```
///
/// # Notes
/// This function ensures thread-safe access and does not panic if
/// the context has been dropped.
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
    "↯"
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
