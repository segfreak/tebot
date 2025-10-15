use std::env;

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
