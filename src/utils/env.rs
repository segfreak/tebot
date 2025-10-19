use std::env;

use teloxide::types::UserId;

use super::parsers;

pub async fn get_token() -> String {
  env::var("BOT_TOKEN").expect("BOT_TOKEN not set")
}

pub async fn get_db_path() -> String {
  let db_path = env::var("DB_PATH").unwrap_or_else(|_| "database.db".to_string());
  db_path
}

pub async fn get_data_dir() -> String {
  let db_path = env::var("DATA_DIR").unwrap_or_else(|_| "data".to_string());
  db_path
}

pub async fn get_prefixes() -> Vec<char> {
  let prefixes: Vec<char> = env::var("PREFIXES")
    .unwrap_or_else(|_| "/".to_string())
    .chars()
    .collect();
  prefixes
}

pub async fn get_owner_id() -> anyhow::Result<UserId> {
  let id_str = env::var("OWNER_ID")?;
  parsers::parse_uid(&id_str).await
}
