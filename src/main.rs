pub mod command;
pub mod config;
pub mod context;
pub mod dispatcher;
pub mod env;
pub mod handler;
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

  let cfg = Config::new_arc_mutex(env::get_token(), env::get_prefixes());
  let _conn_mgr = SqliteConnectionManager::file(env::get_db_path());
  let pool = Arc::new(Pool::new(_conn_mgr).unwrap());
  let perm_mgr = PermissionManager::new_arc_mutex(pool.clone());
  let bot = Arc::new(Bot::new(cfg.lock().await.get_token()));

  let dp = dispatcher::Dispatcher::new_arc_mutex(Weak::new());

  let ctx = Context::new_arc_mutex(
    cfg.clone(),
    pool.clone(),
    perm_mgr.clone(),
    bot.clone(),
    dp.clone(),
  );

  {
    perm_mgr
      .lock()
      .await
      .set(teloxide::prelude::UserId(6737206665), Permission::OWNER);
  }

  {
    dp.lock().await.context = Arc::downgrade(&ctx);
  }

  {
    let plugins_to_register: Vec<plugin::PluginBox> = vec![plugins::core::get_plugin()];

    for plugin in plugins_to_register {
      dp.lock().await.register_plugin(plugin);
    }
  }

  let me = bot.get_me().await?;
  log::info!("logged in as {} [ id: {} ]", me.full_name(), me.id);

  let handler = dptree::entry().endpoint({
    let dp = dp.clone();

    move |update: teloxide::prelude::Update, bot: Arc<Bot>| {
      let dp = dp.clone();

      async move {
        log::debug!("new update received (kind: {:?})", update.kind);
        dp.lock().await.handle_update((*bot).clone(), update).await;
        Ok::<(), teloxide::RequestError>(())
      }
    }
  });

  Dispatcher::builder(bot.clone(), handler)
    .build()
    .dispatch()
    .await;

  Ok(())
}
