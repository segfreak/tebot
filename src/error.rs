use thiserror::Error;

use teloxide::prelude::Requester;

use crate::style::{DefaultStyle, Style};

#[derive(Error, Debug)]
pub enum Error {
  #[error("context is disposed")]
  ContextDisposed,

  #[error(transparent)]
  Teloxide(#[from] teloxide::RequestError),
}

pub async fn emit(
  bot: teloxide::Bot,
  msg: teloxide::prelude::Message,
  err: impl std::error::Error + Send + Sync + 'static,
) -> Box<dyn std::error::Error + Send + Sync> {
  let err_boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(err);
  let _ = bot
    .send_message(
      msg.chat.id,
      format!("{} {}", DefaultStyle::err(), err_boxed),
    )
    .await;
  err_boxed
}
