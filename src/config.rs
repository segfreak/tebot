use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Config {
  pub token: String,
  pub prefixes: Vec<char>,
}

impl Config {
  pub fn new(token: String, prefixes: Vec<char>) -> Self {
    Self {
      token: token,
      prefixes: prefixes,
    }
  }

  pub fn new_arc_mutex(token: String, prefixes: Vec<char>) -> Arc<Mutex<Self>> {
    Arc::new(Mutex::new(Self::new(token, prefixes)))
  }

  pub fn get_token(&self) -> &str {
    &self.token
  }

  pub fn get_prefix(&self) -> &Vec<char> {
    &self.prefixes
  }
}
