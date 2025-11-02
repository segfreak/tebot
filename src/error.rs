use teloxide::prelude::Requester;

use crate::utils::style::{DefaultStyle, Style};

/// Represents the various error types that can occur during
/// bot command execution or general runtime operations.
///
/// This enum provides structured, human-readable errors that
/// can be displayed directly in chat or logged for debugging.
/// It covers most user-facing validation and control errors,
/// while preserving enough context for developers to debug.
///
/// Each variant includes a formatted error message for display.
#[derive(thiserror::Error, Debug)]
pub enum Error {
  /// Indicates that the context object was dropped or became invalid.
  ///
  /// This usually happens when an asynchronous command tries to
  /// access the shared bot state after it has been disposed.
  #[error("context is disposed")]
  ContextDisposed,

  /// Represents a command used with an incorrect number or type of arguments.
  ///
  /// Carries a string describing the correct usage format.
  #[error("usage: {0}")]
  InvalidCommandUsage(String),

  /// Indicates that an invalid value was provided for a command option.
  ///
  /// The associated string specifies which option failed validation.
  #[error("invalid {0}")]
  InvalidOption(String),

  /// Triggered when the specified command name could not be found
  /// in the registered command list.
  #[error("command {0} not found")]
  CommandNotFound(String),

  /// Indicates that a required option or argument was missing.
  ///
  /// The string identifies which parameter was not provided.
  #[error("{0} not specified")]
  OptionNotSpecified(String),

  /// Raised when the user provides an unrecognized or unsupported option.
  ///
  /// Useful for reporting typos or deprecated options.
  #[error("unknown {0}")]
  UnknownOption(String),

  /// Used when a requested resource, entity, or file was not found.
  ///
  /// Includes a description of what specifically was missing.
  #[error("{0} not found")]
  NotFound(String),

  /// Indicates that a given input, list, or file was empty when data was expected.
  ///
  /// The string clarifies which object or argument was empty.
  #[error("{0} is empty")]
  IsEmpty(String),
}

/// Sends a formatted error message to the user and returns the same error object.
///
/// This function is used for graceful error handling within bot commands.
/// It allows reporting the error directly to a chat if both the bot instance
/// and message context are available. Otherwise, it silently returns the error.
///
/// # Arguments
/// * `bot` — Optional [`teloxide::Bot`] instance used to send the message.
/// * `msg` — Optional [`teloxide::Message`] containing chat context.
/// * `err` — Any error convertible into [`anyhow::Error`].
///
/// # Returns
/// The error as an [`anyhow::Error`], suitable for propagation or logging.
///
/// # Example
/// ```rust,ignore
/// if let Err(err) = some_command().await {
///     return Err(emit(Some(bot.clone()), Some(msg.clone()), err).await);
/// }
/// ```
///
/// # Behavior
/// - If both `bot` and `msg` are provided, a message like `"✕ <error>"` is sent.
/// - If either is missing, no message is sent, and the error is just returned.
/// - Uses [`DefaultStyle::s_err`] for visual consistency.
pub async fn emit(
  bot: Option<teloxide::Bot>,
  msg: Option<teloxide::prelude::Message>,
  err: impl Into<anyhow::Error>,
) -> anyhow::Error {
  let _err = err.into();

  if let (Some(bot), Some(msg)) = (bot, msg) {
    let _ = bot
      .send_message(msg.chat.id, format!("{} {:?}", DefaultStyle::s_err(), _err))
      .await;
  }

  _err
}
