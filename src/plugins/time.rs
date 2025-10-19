use chrono::{Local, Utc};
use std::sync::{Arc, Weak};

use indexmap::IndexMap;

use teloxide::payloads::*;
use teloxide::prelude::Requester;
use teloxide::types::Message;
use teloxide::Bot;

use crate::bot::command::{self, CommandMetadata, ReplyRequirement};
use crate::permissions::types::Permission;

use crate::{
  bot::{context, handler, plugin},
  utils::{formatter, style},
};

async fn on_uptime(
  _bot: Bot,
  _msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let _style = style::get_style(_ctx.clone()).await;
  let _uptime = crate::START_TIME.elapsed();
  let _formatted_uptime = formatter::format_duration(_uptime);
  let _msg_text = format!(
    "{} <b>Uptime</b>: <b>{}</b>",
    _style.arrow(),
    _formatted_uptime
  );
  let _ = _bot
    .send_message(_msg.chat.id, _msg_text)
    .parse_mode(teloxide::types::ParseMode::Html)
    .await;
  Ok(())
}

async fn on_datetime(
  _bot: Bot,
  _msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let _style = style::get_style(_ctx.clone()).await;

  let _utc_now = Utc::now();
  let _local_now = Local::now();

  let _utc_formatted = _utc_now.format("%Y-%m-%d %H:%M:%S UTC");
  let _local_formatted = _local_now.format("%Y-%m-%d %H:%M:%S %Z");

  let _timestamp = _utc_now.timestamp();

  let _iso_formatted = _utc_now.to_rfc3339();

  let _msg_text = format!(
    "{} <b>Date & Time</b>\n\n\
    {} <b>UTC</b>: <code>{}</code>\n\
    {} <b>Local</b>: <code>{}</code>\n\
    {} <b>Unix</b>: <code>{}</code>\n\
    {} <b>ISO 8601</b>: <code>{}</code>",
    _style.ok(),
    _style.bullet(),
    _utc_formatted,
    _style.bullet(),
    _local_formatted,
    _style.bullet(),
    _timestamp,
    _style.bullet(),
    _iso_formatted
  );

  let _ = _bot
    .send_message(_msg.chat.id, _msg_text)
    .parse_mode(teloxide::types::ParseMode::Html)
    .await;

  Ok(())
}

pub struct Plugin {}

impl Plugin {
  pub fn new() -> Self {
    Self {}
  }
}

impl plugin::Plugin for Plugin {
  fn name(&self) -> &str {
    "time"
  }

  fn commands(&self) -> indexmap::IndexMap<String, command::CommandMetadata> {
    let mut cmds = IndexMap::new();

    let uptime_cmd = CommandMetadata::new(
      Permission::USER,
      "Display bot uptime since last restart".to_string(),
      ReplyRequirement::None,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_uptime(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    let datetime_cmd = CommandMetadata::new(
      Permission::USER,
      "Display current date and time in multiple formats".to_string(),
      ReplyRequirement::None,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_datetime(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    cmds.insert("uptime".to_string(), uptime_cmd);
    cmds.insert("datetime".to_string(), datetime_cmd);

    cmds
  }

  fn update_handlers(&self) -> Vec<handler::UpdateHandler> {
    Vec::new()
  }
}

pub fn get_plugin() -> plugin::PluginBox {
  Box::new(Plugin::new())
}
