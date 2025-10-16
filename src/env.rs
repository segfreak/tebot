use std::env;

use teloxide::types::UserId;

use super::parsers;

pub fn get_token() -> String {
  env::var("BOT_TOKEN").expect("BOT_TOKEN not set")
}

pub fn get_db_path() -> String {
  let db_path = env::var("DB_PATH").unwrap_or_else(|_| "database.db".to_string());
  db_path
}

pub fn get_prefixes() -> Vec<char> {
  let prefixes: Vec<char> = env::var("PREFIXES")
    .unwrap_or_else(|_| "/".to_string())
    .chars()
    .collect();
  prefixes
}

pub fn get_owner_id() -> Result<UserId, String> {
  let id_str =
    env::var("OWNER_ID").map_err(|_| "Missing OWNER_ID environment variable".to_string())?;
  parsers::parse_uid(&id_str)
}
