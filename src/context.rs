use std::sync::{Arc, Mutex};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use super::command::CommandDispatcher;
use super::config::Config;
use super::permissions::PermissionManager;
use super::plugin::PluginCommandDispatcher;

#[derive(Debug)]
pub struct Context {
  pub cfg: Arc<Mutex<Config>>,
  pub db: Arc<Pool<SqliteConnectionManager>>,
  pub perm_mgr: Arc<Mutex<PermissionManager>>,
  pub bot: Arc<teloxide::Bot>,

  pub cmd_dp: Arc<Mutex<CommandDispatcher>>,
  pub plug_cmd_dp: Arc<Mutex<PluginCommandDispatcher>>,
}

impl Context {
  pub fn new(
    cfg: Arc<Mutex<Config>>,
    db: Arc<Pool<SqliteConnectionManager>>,
    perm_mgr: Arc<Mutex<PermissionManager>>,
    bot: Arc<teloxide::Bot>,
    cmd_dp: Arc<Mutex<CommandDispatcher>>,
    plug_cmd_dp: Arc<Mutex<PluginCommandDispatcher>>,
  ) -> Self {
    Self {
      cfg: cfg,
      db: db,
      perm_mgr: perm_mgr,
      bot: bot,
      cmd_dp: cmd_dp,
      plug_cmd_dp: plug_cmd_dp,
    }
  }

  pub fn new_arc_mutex(
    cfg: Arc<Mutex<Config>>,
    db: Arc<r2d2::Pool<SqliteConnectionManager>>,
    perm_mgr: Arc<Mutex<PermissionManager>>,
    bot: Arc<teloxide::Bot>,
    cmd_dp: Arc<Mutex<CommandDispatcher>>,
    plug_cmd_dp: Arc<Mutex<PluginCommandDispatcher>>,
  ) -> Arc<Mutex<Self>> {
    Arc::new(Mutex::new(Self::new(
      cfg,
      db,
      perm_mgr,
      bot,
      cmd_dp,
      plug_cmd_dp,
    )))
  }
}
