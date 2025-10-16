pub mod command;
pub mod config;
pub mod context;
pub mod dispatcher;
pub mod env;
pub mod handler;
pub mod parsers;
pub mod permissions;
pub mod plugin;

pub mod plugins;

use config::Config;
use context::Context;
use permissions::PermissionManager;

use teloxide::{
  dptree,
  prelude::{Dispatcher, Requester},
  Bot,
};

use std::sync::{Arc, Weak};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use dotenvy::dotenv;

use crate::permissions::Permission;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dotenv().ok();
  env_logger::init();

  log::trace!("initializing configuration");
  let cfg = Config::new_arc_mutex(env::get_token(), env::get_prefixes());

  log::trace!("setting up database connection pool");
  let _conn_mgr = SqliteConnectionManager::file(env::get_db_path());
  let pool = Arc::new(Pool::new(_conn_mgr).unwrap());

  log::trace!("initializing permission manager");
  let perm_mgr = PermissionManager::new_arc_mutex(pool.clone());

  log::trace!("creating bot instance");
  let bot = Arc::new(Bot::new(cfg.lock().await.get_token()));

  log::trace!("creating dispatcher");
  let dp = dispatcher::Dispatcher::new_arc_mutex(Weak::new());

  log::trace!("creating context");
  let ctx = Context::new_arc_mutex(
    cfg.clone(),
    pool.clone(),
    perm_mgr.clone(),
    bot.clone(),
    dp.clone(),
  );

  {
    if let Ok(owner_id) = env::get_owner_id() {
      perm_mgr.lock().await.set(owner_id, Permission::OWNER);
    }
  }

  {
    log::trace!("linking dispatcher to context");
    dp.lock().await.context = Arc::downgrade(&ctx);
  }

  {
    let plugins_to_register: Vec<plugin::PluginBox> = vec![plugins::core::get_plugin()];

    for plugin in plugins_to_register {
      dp.lock().await.register_plugin(plugin);
    }
  }

  let me = bot.get_me().await?;
  log::info!("bot logged in as {} [id: {}]", me.full_name(), me.id);

  let handler = dptree::entry().endpoint({
    let dp = dp.clone();

    move |update: teloxide::prelude::Update, bot: Arc<Bot>| {
      let dp = dp.clone();

      async move {
        log::trace!("new update received, kind: {:?}", update.kind);
        dp.lock().await.handle_update((*bot).clone(), update).await;
        Ok::<(), teloxide::RequestError>(())
      }
    }
  });

  log::trace!("starting dispatcher");
  Dispatcher::builder(bot.clone(), handler)
    .build()
    .dispatch()
    .await;

  log::trace!("shutdown complete");
  Ok(())
}
