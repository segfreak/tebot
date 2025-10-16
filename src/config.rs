use std::sync::Arc;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
  pub token: String,
  pub prefixes: Vec<char>,
}

impl Config {
  pub fn default() -> Self {
    log::trace!("creating default config with empty token and '/' prefix");
    Self {
      token: String::new(),
      prefixes: vec!['/'],
    }
  }

  pub fn default_arc_mutex() -> Arc<Mutex<Self>> {
    log::trace!("wrapping default config in arc mutex");
    Arc::new(Mutex::new(Self::default()))
  }

  pub fn new(token: String, prefixes: Vec<char>) -> Self {
    log::trace!(
      "creating new config with token='{}' and prefixes={:?}",
      token,
      prefixes
    );
    Self { token, prefixes }
  }

  pub fn new_arc_mutex(token: String, prefixes: Vec<char>) -> Arc<Mutex<Self>> {
    log::trace!("wrapping new config in arc mutex");
    Arc::new(Mutex::new(Self::new(token, prefixes)))
  }

  pub fn get_token(&self) -> &str {
    log::trace!("retrieving config token");
    &self.token
  }

  pub fn get_prefixes(&self) -> Vec<char> {
    log::trace!("retrieving config prefixes: {:?}", self.prefixes);
    self.prefixes.clone()
  }
}
