use std::env;

use teloxide::types::UserId;

use super::parsers;

/// Returns the Telegram bot token from the environment.
///
/// This token is required for authenticating with the Telegram Bot API.
/// The function will panic if `BOT_TOKEN` is not set in the environment.
///
/// It is recommended to define this variable in a `.env` file
/// or system-wide environment configuration.
pub async fn get_token() -> String {
  env::var("BOT_TOKEN").expect("BOT_TOKEN not set")
}

/// Returns the path to the bot’s database file.
///
/// If the `DB_PATH` environment variable is not set, a default value
/// of `"database.db"` is used.  
///
/// This path defines where the bot stores persistent data such as user
/// states or configuration.
pub async fn get_db_path() -> String {
  let db_path = env::var("DB_PATH").unwrap_or_else(|_| "database.db".to_string());
  db_path
}

/// Returns the directory used for storing persistent bot data.
///
/// The directory is read from the `DATA_DIR` environment variable,
/// falling back to `"data"` if unset.  
///
/// It serves as the root for all file operations, such as plugin data
/// and temporary files.
pub async fn get_data_dir() -> String {
  let db_path = env::var("DATA_DIR").unwrap_or_else(|_| "data".to_string());
  db_path
}

/// Returns a list of command prefixes from the environment.
///
/// Command prefixes define which symbols can trigger bot commands
/// (for example, `/`, `!`, or `.`).  
///
/// If not specified via `PREFIXES`, this function defaults to `/`.
pub async fn get_prefixes() -> Vec<char> {
  let prefixes: Vec<char> = env::var("PREFIXES")
    .unwrap_or_else(|_| "/".to_string())
    .chars()
    .collect();
  prefixes
}

/// Returns the bot owner’s Telegram user ID.
///
/// This value is read from the `OWNER_ID` environment variable
/// and parsed into a [`UserId`].  
///
/// # Errors
/// Returns an error if the variable is missing or cannot be parsed.
pub async fn get_owner_id() -> anyhow::Result<UserId> {
  let id_str = env::var("OWNER_ID")?;
  parsers::parse_uid(&id_str).await
}
