use std::sync::{Arc, Mutex};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use super::config::Config;
use super::permissions::PermissionManager;

#[derive(Debug)]
pub struct Context {
  pub cfg: Arc<Mutex<Config>>,
  pub db: Arc<Pool<SqliteConnectionManager>>,
  pub perm_mgr: Arc<Mutex<PermissionManager>>,
  pub bot: Arc<teloxide::Bot>,
}

impl Context {
  pub fn new(
    cfg: Arc<Mutex<Config>>,
    db: Arc<Pool<SqliteConnectionManager>>,
    perm_mgr: Arc<Mutex<PermissionManager>>,
    bot: Arc<teloxide::Bot>,
  ) -> Self {
    Self {
      cfg: cfg,
      db: db,
      perm_mgr: perm_mgr,
      bot: bot,
    }
  }

  pub fn new_arc_mutex(
    cfg: Arc<Mutex<Config>>,
    db: Arc<r2d2::Pool<SqliteConnectionManager>>,
    perm_mgr: Arc<Mutex<PermissionManager>>,
    bot: Arc<teloxide::Bot>,
  ) -> Arc<Mutex<Self>> {
    Arc::new(Mutex::new(Self::new(cfg, db, perm_mgr, bot)))
  }
}
