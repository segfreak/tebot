pub mod error;

pub mod bot;
pub mod permissions;
pub mod utils;

pub mod plugins;

use crate::{
  permissions::{manager::PermissionManager, types::Permission},
  utils::editor::Editor,
};

use bot::{config::Config, context::Context, dispatcher, plugin};

use teloxide::{
  dptree,
  prelude::{Dispatcher, Requester},
  Bot,
};
use tokio::sync::Mutex;

use std::{
  fs::OpenOptions,
  io::Write,
  sync::{Arc, Weak},
};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use dotenvy::dotenv;
use once_cell::sync::Lazy;
use std::time::Instant;

use clap::{Parser, Subcommand};

pub static START_TIME: Lazy<Instant> = Lazy::new(|| {
  log::trace!("initializing start time");
  Instant::now()
});

#[derive(Parser)]
#[command(name = "tebot")]
#[command(about = "Telegram bot runtime", long_about = None)]
#[command(version)]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
  /// Runs telegram bot
  Run,
  /// Opens the .env configuration file in a text editor
  Configure,
}

fn init() {
  // Environment initializing
  dotenv().ok();

  // Logging initializing
  env_logger::init();

  // Start time initializing
  // Accessing START_TIME here ensures it is initialized,
  // because Lazy is only evaluated on first use.
  let _ = START_TIME.elapsed();
}

async fn run() -> anyhow::Result<()> {
  let cfg = Config::new_shared(
    utils::env::get_token().await,
    utils::env::get_prefixes().await,
  );
  let _conn_mgr = SqliteConnectionManager::file(utils::env::get_db_path().await);

  let pool = Arc::new(Pool::new(_conn_mgr)?);

  let perm_mgr = PermissionManager::new_shared(pool.clone())?;
  let bot = Arc::new(Bot::new(cfg.lock().await.get_token()));
  let dp = dispatcher::Dispatcher::new_shared(Weak::new());
  let style = Arc::new(utils::style::DefaultStyle);
  let ctx = Arc::new(Mutex::new(Context::new(
    cfg.clone(),
    pool.clone(),
    perm_mgr.clone(),
    bot.clone(),
    dp.clone(),
    style.clone(),
  )));

  {
    if let Ok(owner_id) = utils::env::get_owner_id().await {
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  init();

  let cli = Cli::parse();

  match cli.command.unwrap_or(Commands::Run) {
    Commands::Configure => {
      let _env_path = ".env";

      if !std::path::Path::new(_env_path).exists() {
        let _def_env = r#"# === Telegram Bot Configuration ===
BOT_TOKEN=your_bot_token_here
OWNER_ID=your_user_id_here

# === Database & Data Paths ===
DB_PATH=./database.db
DATA_DIR=./data

# === Command Settings ===
PREFIXES=/

# === Logging ===
RUST_LOG=info
"#;

        let mut file = OpenOptions::new()
          .create(true)
          .write(true)
          .truncate(true)
          .open(_env_path)?;
        file.write_all(_def_env.as_bytes())?;

        log::trace!("created default .env");
      }

      let _editor = Editor::detect()?;
      _editor.open_file(_env_path, true)?;
    }

    Commands::Run => {
      log::trace!("running bot entrypoint");
      return run().await;
    }
  }

  Ok(())
}
