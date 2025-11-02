use derivative::Derivative;
use std::sync::Arc;
use tokio::sync::Mutex;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::utils::style::{self, DynStyle};

use super::config::Config;
use super::dispatcher::Dispatcher;

use crate::permissions::manager::PermissionManager;

/// Represents the shared context for the bot.
///
/// This structure holds all global resources that are shared across
/// commands, handlers, and asynchronous tasks. It includes configuration,
/// database pool, permission manager, the bot instance, dispatcher, and
/// styling utilities.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Context {
  /// Shared configuration for the bot.
  pub cfg: Arc<Mutex<Config>>,

  /// SQLite connection pool wrapped in `r2d2` for concurrent access.
  pub db: Arc<Pool<SqliteConnectionManager>>,

  /// Permission manager wrapped in a `Mutex` for async-safe access.
  pub perm_mgr: Arc<Mutex<PermissionManager>>,

  /// Telegram bot instance used to send and receive messages.
  pub bot: Arc<teloxide::Bot>,

  /// Dispatcher for managing commands and message handling.
  pub dp: Arc<Mutex<Dispatcher>>,

  /// Dynamic style for formatting messages and logs.
  ///
  /// Stored as a trait object to allow customization of output styles.
  #[derivative(Debug = "ignore")]
  pub style: Arc<dyn DynStyle>,
}

impl Context {
  /// Creates a new [`Context`] instance with the given resources.
  ///
  /// # Parameters
  /// - `cfg`: Shared bot configuration.
  /// - `db`: Database connection pool.
  /// - `perm_mgr`: Shared permission manager.
  /// - `bot`: Telegram bot instance.
  /// - `dp`: Dispatcher instance.
  /// - `style`: Dynamic style for message formatting.
  pub fn new(
    cfg: Arc<Mutex<Config>>,
    db: Arc<Pool<SqliteConnectionManager>>,
    perm_mgr: Arc<Mutex<PermissionManager>>,
    bot: Arc<teloxide::Bot>,
    dp: Arc<tokio::sync::Mutex<Dispatcher>>,
    style: Arc<dyn DynStyle>,
  ) -> Self {
    Self {
      cfg,
      db,
      perm_mgr,
      bot,
      dp,
      style,
    }
  }
}
