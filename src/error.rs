use teloxide::prelude::Requester;

use crate::utils::style::{DefaultStyle, Style};

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("context is disposed")]
  ContextDisposed,
  // #[error(transparent)]
  // Teloxide(#[from] teloxide::RequestError),
}

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
