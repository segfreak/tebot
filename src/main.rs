pub mod command;
pub mod config;
pub mod context;
pub mod env;
pub mod permissions;
pub mod plugin;

pub mod plugins;

use command::CommandDispatcher;
use config::Config;
use context::Context;
use permissions::PermissionManager;
use plugin::PluginCommandDispatcher;

use teloxide::{
  dispatching::UpdateFilterExt,
  prelude::{Dispatcher, Message, Requester},
  Bot,
};

use std::sync::{Arc, Weak};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  dotenv().ok();
  env_logger::init();

  let cfg = Config::new_arc_mutex(env::get_token(), env::get_prefixes());
  let _conn_mgr = SqliteConnectionManager::file(env::get_db_path());
  let pool = Arc::new(Pool::new(_conn_mgr).unwrap());
  let perm_mgr = PermissionManager::new_arc_mutex(pool.clone());
  let bot = Arc::new(Bot::new(cfg.lock().unwrap().get_token()));

  let cmd_dp = CommandDispatcher::new_arc_mutex(Weak::new());
  let plug_cmd_dp = PluginCommandDispatcher::new_arc_mutex(Weak::new());

  let ctx = Context::new_arc_mutex(
    cfg.clone(),
    pool.clone(),
    perm_mgr.clone(),
    bot.clone(),
    cmd_dp.clone(),
    plug_cmd_dp.clone(),
  );

  {
    cmd_dp.lock().await.context = Arc::downgrade(&ctx);
  }

  {
    plug_cmd_dp.lock().await.context = Arc::downgrade(&ctx);
  }

  {
    let plugins_to_register: Vec<plugin::PluginBox> = vec![plugins::core::get_plugin()];

    for plugin in plugins_to_register {
      plug_cmd_dp.lock().await.register_plugin(plugin);
    }
  }

  let me = bot.get_me().await?;
  log::info!("logged in as {} [ id: {} ]", me.full_name(), me.id);
  log::trace!("context dump:\n{:#?}", ctx);

  let handler = teloxide::prelude::Update::filter_message().endpoint({
    let dp = cmd_dp.clone();
    let pdp = plug_cmd_dp.clone();

    move |msg: Message, bot: Arc<Bot>| {
      let dp = dp.clone();
      let pdp = pdp.clone();

      async move {
        dp.lock()
          .await
          .handle_message((*bot).clone(), msg.clone())
          .await;
        pdp.lock().await.handle_message((*bot).clone(), msg).await;

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
