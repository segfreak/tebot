pub mod command;
pub mod config;
pub mod context;
pub mod permissions;

use command::CommandRegistry;
use config::Config;
use context::Context;
use permissions::PermissionManager;

use teloxide::{prelude::Requester, Bot};

use std::sync::{Arc, Weak};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  env_logger::init();

  let cfg = Config::new_arc_mutex("token".to_string(), vec!['!', '/']);
  let _conn_mgr = SqliteConnectionManager::file("database.db");
  let pool = Arc::new(Pool::new(_conn_mgr).unwrap());
  let perm_mgr = PermissionManager::new_arc_mutex(pool.clone());
  let bot = Arc::new(Bot::new(cfg.lock().unwrap().get_token()));
  let cmd_reg = CommandRegistry::new_arc_mutex(Weak::new());
  let ctx = Context::new_arc_mutex(
    cfg.clone(),
    pool.clone(),
    perm_mgr.clone(),
    bot.clone(),
    cmd_reg.clone(),
  );

  {
    cmd_reg.lock().unwrap().context = Arc::downgrade(&ctx);
  }

  let me = bot.get_me().await?;
  log::info!("logged in as {} [ id: {} ]", me.full_name(), me.id);

  println!("{:#?}", ctx);

  Ok(())
}
