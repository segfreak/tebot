pub mod command;
pub mod config;
pub mod context;
pub mod dispatcher;
pub mod env;
pub mod error;
pub mod formatter;
pub mod handler;
pub mod metadata;
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
use tokio::sync::Mutex;

use std::sync::{Arc, Weak};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use dotenvy::dotenv;
use once_cell::sync::Lazy;
use std::time::Instant;

use crate::permissions::Permission;

pub static START_TIME: Lazy<Instant> = Lazy::new(|| {
  log::debug!("initializing start time");
  Instant::now()
});

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenv().ok();
  env_logger::init();

  // Accessing START_TIME here ensures it is initialized,
  // because Lazy is only evaluated on first use.
  let _ = START_TIME.elapsed();

  let cfg = Config::new_shared(env::get_token().await, env::get_prefixes().await);
  let _conn_mgr = SqliteConnectionManager::file(env::get_db_path().await);

  let pool = Arc::new(Pool::new(_conn_mgr)?);

  let perm_mgr = PermissionManager::new_shared(pool.clone())?;
  let bot = Arc::new(Bot::new(cfg.lock().await.get_token()));
  let dp = dispatcher::Dispatcher::new_shared(Weak::new());
  let style = Arc::new(style::DefaultStyle);
  let ctx = Arc::new(Mutex::new(Context::new(
    cfg.clone(),
    pool.clone(),
    perm_mgr.clone(),
    bot.clone(),
    dp.clone(),
    style.clone(),
  )));

  {
    if let Ok(owner_id) = env::get_owner_id().await {
      perm_mgr.lock().await.set(owner_id, Permission::OWNER)?;
    }
  }

  {
    dp.lock().await.context = Arc::downgrade(&ctx);
  }

  {
    plugin::register_all(dp.clone(), plugins::all().await).await;
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

  Ok(())
}
