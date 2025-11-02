use std::sync::Arc;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};

/// Represents the configuration for the bot.
///
/// This structure typically contains:
/// - `token`: The bot token used for authentication with the Telegram API.
/// - `prefixes`: A list of allowed command prefixes (e.g., `'/'`, `'!'`).
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
  /// Telegram bot token.
  pub token: String,

  /// Allowed command prefixes for parsing commands from messages.
  pub prefixes: Vec<char>,
}

impl Config {
  /// Returns a default [`Config`] instance.
  ///
  /// Defaults to an empty token and a single `'/'` prefix.
  ///
  /// # Examples
  /// ```
  /// use crate::config::Config;
  /// let cfg = Config::default();
  /// assert_eq!(cfg.token, "");
  /// assert_eq!(cfg.prefixes, vec!['/']);
  /// ```
  pub fn default() -> Self {
    Self {
      token: String::new(),
      prefixes: vec!['/'],
    }
  }

  /// Returns a default [`Config`] wrapped in an `Arc<Mutex<_>>`.
  ///
  /// Useful for sharing the configuration safely across async tasks.
  ///
  /// # Examples
  /// ```
  /// use crate::config::Config;
  /// use std::sync::Arc;
  /// use tokio::sync::Mutex;
  /// let cfg = Config::default_arc_mutex();
  /// ```
  pub fn default_arc_mutex() -> Arc<Mutex<Self>> {
    Arc::new(Mutex::new(Self::default()))
  }

  /// Creates a new [`Config`] with the given token and prefixes.
  ///
  /// # Parameters
  /// - `token`: Bot token for Telegram authentication.
  /// - `prefixes`: List of allowed command prefixes.
  ///
  /// # Examples
  /// ```
  /// use crate::config::Config;
  /// let cfg = Config::new("my_token".to_string(), vec!['/', '!']);
  /// ```
  pub fn new(
    token: String,
    prefixes: Vec<char>,
  ) -> Self {
    Self { token, prefixes }
  }

  /// Creates a new [`Config`] wrapped in `Arc<Mutex<_>>` for shared access.
  ///
  /// # Examples
  /// ```
  /// use crate::config::Config;
  /// let shared_cfg = Config::new_shared("token".to_string(), vec!['/']);
  /// ```
  pub fn new_shared(
    token: String,
    prefixes: Vec<char>,
  ) -> Arc<Mutex<Self>> {
    Arc::new(Mutex::new(Self::new(token, prefixes)))
  }

  /// Returns a reference to the bot token.
  ///
  /// # Examples
  /// ```
  /// use crate::config::Config;
  /// let cfg = Config::default();
  /// let token = cfg.get_token();
  /// ```
  pub fn get_token(&self) -> &str {
    &self.token
  }

  /// Returns a clone of the command prefixes vector.
  ///
  /// Cloning ensures that the internal configuration cannot be modified
  /// unintentionally from outside.
  ///
  /// # Examples
  /// ```
  /// use crate::config::Config;
  /// let cfg = Config::default();
  /// let prefixes = cfg.get_prefixes();
  /// ```
  pub fn get_prefixes(&self) -> Vec<char> {
    self.prefixes.clone()
  }
}
