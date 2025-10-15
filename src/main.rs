pub mod command;
pub mod config;
pub mod context;
pub mod permissions;

use config::Config;
use context::Context;
use permissions::PermissionManager;

use std::sync::Arc;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

fn main() {
  let cfg = Config::new_arc_mutex("token".to_string(), vec!['!', '/']);
  let _conn_mgr = SqliteConnectionManager::file("database.db");
  let pool = Arc::new(Pool::new(_conn_mgr).unwrap());
  let perm_mgr = PermissionManager::new_arc_mutex(pool.clone());
  let ctx = Context::new_arc_mutex(cfg.clone(), pool.clone(), perm_mgr.clone());

  println!("{:#?}", ctx);
}
