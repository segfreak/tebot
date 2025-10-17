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
    Self {
      token: String::new(),
      prefixes: vec!['/'],
    }
  }

  pub fn default_arc_mutex() -> Arc<Mutex<Self>> {
    Arc::new(Mutex::new(Self::default()))
  }

  pub fn new(token: String, prefixes: Vec<char>) -> Self {
    Self { token, prefixes }
  }

  pub fn new_shared(token: String, prefixes: Vec<char>) -> Arc<Mutex<Self>> {
    Arc::new(Mutex::new(Self::new(token, prefixes)))
  }

  pub fn get_token(&self) -> &str {
    &self.token
  }

  pub fn get_prefixes(&self) -> Vec<char> {
    self.prefixes.clone()
  }
}
