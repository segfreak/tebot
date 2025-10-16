use std::pin::Pin;
use std::sync::{Arc, Mutex, Weak};

use super::command;
use super::context;

pub type MessageHandler = Arc<
  dyn Fn(teloxide::Bot, teloxide::prelude::Message, Weak<Mutex<context::Context>>) + Send + Sync,
>;

pub type CommandHandler = Arc<
  dyn Fn(teloxide::Bot, teloxide::prelude::Message, command::Command, Weak<Mutex<context::Context>>)
    + Send
    + Sync,
>;

pub type UpdateHandler = Arc<
  dyn Fn(
      teloxide::Bot,
      teloxide::prelude::Update,
      Weak<Mutex<context::Context>>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>
    + Send
    + Sync,
>;
