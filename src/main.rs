pub mod command;
pub mod config;
pub mod context;
pub mod dispatcher;
pub mod env;
pub mod error;
pub mod handler;
pub mod parsers;
pub mod permissions;
pub mod plugin;
pub mod style;

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
async fn main() -> anyhow::Result<()> {
  dotenv().ok();
  env_logger::init();

  let cfg = Config::new_arc_mutex(env::get_token(), env::get_prefixes());
  let _conn_mgr = SqliteConnectionManager::file(env::get_db_path());

  let pool = Arc::new(Pool::new(_conn_mgr)?);

  let perm_mgr = PermissionManager::new_arc_mutex(pool.clone())?;
  let bot = Arc::new(Bot::new(cfg.lock().await.get_token()));
  let dp = dispatcher::Dispatcher::new_arc_mutex(Weak::new());
  let style = Arc::new(style::DefaultStyle);
  let ctx = Context::new_arc_mutex(
    cfg.clone(),
    pool.clone(),
    perm_mgr.clone(),
    bot.clone(),
    dp.clone(),
    style.clone(),
  );

  {
    if let Ok(owner_id) = env::get_owner_id() {
      perm_mgr.lock().await.set(owner_id, Permission::OWNER)?;
    }
  }

  {
    dp.lock().await.context = Arc::downgrade(&ctx);
  }

  {
    plugin::register_all(dp.clone(), plugins::all()).await;
  }

  let me = bot.get_me().await?;
  log::info!("bot logged in as {} [id: {}]", me.full_name(), me.id);

  let handler = dptree::entry().endpoint({
    let dp = dp.clone();

    move |update: teloxide::prelude::Update, bot: Arc<Bot>| {
      let dp = dp.clone();

      async move {
        log::trace!("new update received, kind: {:?}", update.kind);

        if let Err(err) = dp.lock().await.handle_update((*bot).clone(), update).await {
          log::error!("error handling update: {:?}", err);
        }

        Ok::<(), teloxide::RequestError>(())
      }
    }
  });

  log::trace!("starting dispatcher");
  Dispatcher::builder(bot.clone(), handler)
    .build()
    .dispatch()
    .await;

  log::trace!("shutdown");
  Ok(())
}
