use std::pin::Pin;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

use super::command;
use super::context;

/// Type alias for a synchronous message handler.
///
/// Receives the bot, the message, and a weak reference to the context.
pub type MessageHandler = Arc<
  dyn Fn(teloxide::Bot, teloxide::prelude::Message, Weak<Mutex<context::Context>>) + Send + Sync,
>;

/// Type alias for a synchronous command handler.
///
/// Receives the bot, the message, the parsed command, and a weak reference to the context.
pub type CommandHandler = Arc<
  dyn Fn(teloxide::Bot, teloxide::prelude::Message, command::Command, Weak<Mutex<context::Context>>)
    + Send
    + Sync,
>;

/// Type alias for an asynchronous update handler.
///
/// Receives the bot, the update, and a weak reference to the context.
/// Returns a pinned future for async execution.
pub type UpdateHandler = Arc<
  dyn Fn(
      teloxide::Bot,
      teloxide::prelude::Update,
      Weak<Mutex<context::Context>>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>>
    + Send
    + Sync,
>;
