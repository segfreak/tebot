use derivative::Derivative;
use std::sync::Arc;
use tokio::sync::Mutex;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::style::DynStyle;

use super::config::Config;
use super::dispatcher::Dispatcher;
use super::permissions::PermissionManager;
use super::style;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Context {
  pub cfg: Arc<Mutex<Config>>,
  pub db: Arc<Pool<SqliteConnectionManager>>,
  pub perm_mgr: Arc<Mutex<PermissionManager>>,
  pub bot: Arc<teloxide::Bot>,

  pub dp: Arc<tokio::sync::Mutex<Dispatcher>>,

  #[derivative(Debug = "ignore")]
  pub style: Arc<dyn style::DynStyle>,
}

impl Context {
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

  pub fn new_shared(
    cfg: Arc<Mutex<Config>>,
    db: Arc<Pool<SqliteConnectionManager>>,
    perm_mgr: Arc<Mutex<PermissionManager>>,
    bot: Arc<teloxide::Bot>,
    dp: Arc<tokio::sync::Mutex<Dispatcher>>,
    style: Arc<dyn DynStyle>,
  ) -> Arc<Mutex<Self>> {
    Arc::new(Mutex::new(Self::new(cfg, db, perm_mgr, bot, dp, style)))
  }
}
