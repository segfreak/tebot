use std::sync::{Arc, Weak};
use std::time::Duration;
use thiserror::Error;

use indexmap::IndexMap;

use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{Message, UserId};
use teloxide::Bot;

use crate::command::{self, ArgMetadata, ArgRequirement, CommandMetadata, ReplyRequirement};
use crate::permissions::Permission;
use crate::{context, error, plugin, style};

#[derive(Error, Debug)]
pub enum CoreError {
  #[error(transparent)]
  Internal(#[from] error::Error),

  #[error("usage: {0}")]
  InvalidCommandUsage(String),

  #[error("invalid {0}")]
  InvalidOption(String),

  #[error("command {0} not found")]
  CommandNotFound(String),

  #[error("{0} not specified")]
  OptionNotSpecified(String),

  #[error("unknown {0}")]
  UnknownOption(String),

  #[error("{0} not found")]
  NotFound(String),

  #[error("{0} is empty")]
  IsEmpty(String),
}

async fn on_id(
  bot: Bot,
  msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let chat_id = msg.chat.id;

  let _style = style::get_style(_ctx.clone()).await;

  let mut text = String::new();

  text.push_str(&format!("‣ <b>Chat ID</b>: <code>{}</code>\n", chat_id));

  if let Some(reply) = msg.reply_to_message() {
    let replied_user_id = reply.from.as_ref().map(|u| u.id).unwrap_or(UserId(0));
    text.push_str(&format!(
      "{} <b>User ID</b>: <code>{}</code>\n",
      _style.bullet(),
      replied_user_id
    ));
  }

  let _ = bot
    .send_message(chat_id, text)
    .parse_mode(teloxide::types::ParseMode::Html)
    .await;

  Ok(())
}

async fn on_help(
  bot: Bot,
  msg: Message,
  cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let chat_id = msg.chat.id;

  let ctx = match _ctx.upgrade() {
    Some(ctx) => ctx,
    None => {
      return Err(
        error::emit(
          Some(bot.clone()),
          Some(msg.clone()),
          error::Error::ContextDisposed,
        )
        .await,
      )
    }
  };

  let style = style::get_style(_ctx.clone()).await;

  let ctx_guard = ctx.lock().await;
  let dp_guard = ctx_guard.dp.lock().await;
  let prefix = ctx_guard.cfg.lock().await.get_prefixes()[0];

  let help_text = if let Some(command_name) = cmd.args.get(0) {
    if let Some(info) = dp_guard.command_handlers.get(command_name) {
      let args_desc: Vec<String> = info
        .args
        .iter()
        .map(|arg| {
          format!(
            "{} {} [{}] → {}",
            style.arrow(),
            arg.name,
            format!("{:?}", arg.requirement),
            arg.description
          )
        })
        .collect();

      format!(
        "{} Command: <code>{}{}</code>\n\
      {} Permission: <b>{:?}</b>\n\
      {} Description: {}\n\
      {} Arguments:\n{}\n\
      {} Reply: <i>{:?}</i>",
        style.ok(),
        prefix,
        command_name,
        style.info(),
        info.perm,
        style.info(),
        info.desc,
        style.info(),
        args_desc.join(&format!("\n")),
        style.info(),
        info.reply
      )
    } else {
      return Err(
        error::emit(
          Some(bot.clone()),
          Some(msg.clone()),
          CoreError::CommandNotFound(command_name.to_string()),
        )
        .await,
      );
    }
  } else {
    let mut sections = Vec::new();

    for (plugin_name, plugin) in &dp_guard.plugins {
      let commands_list: Vec<String> = plugin
        .commands()
        .iter()
        .map(|(name, info)| {
          format!(
            "{} <code>{}{}</code> → {}",
            style.info(),
            prefix,
            name,
            info.desc
          )
        })
        .collect();

      let section = format!(
        "{} {}:\n{}",
        style.bullet(),
        plugin_name,
        commands_list.join("\n")
      );
      sections.push(section);
    }

    sections.join("\n\n")
  };

  let _ = bot
    .send_message(chat_id, help_text)
    .parse_mode(teloxide::types::ParseMode::Html)
    .await;

  Ok(())
}

fn format_duration(dur: Duration) -> String {
  let secs = dur.as_secs();
  let hours = secs / 3600;
  let minutes = (secs % 3600) / 60;
  let seconds = secs % 60;

  if hours > 0 {
    format!("{}h {}m {}s", hours, minutes, seconds)
  } else if minutes > 0 {
    format!("{}m {}s", minutes, seconds)
  } else {
    format!("{}s", seconds)
  }
}

async fn on_uptime(
  _bot: Bot,
  _msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let _style = style::get_style(_ctx.clone()).await;
  let _uptime = crate::START_TIME.elapsed();
  let _formatted_uptime = format_duration(_uptime);
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

async fn on_shutdown(
  _bot: Bot,
  _msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let _style = style::get_style(_ctx.clone()).await;
  let _msg_text = format!("{} Shuting down...", _style.arrow());
  let _ = _bot
    .send_message(_msg.chat.id, _msg_text)
    .parse_mode(teloxide::types::ParseMode::Html)
    .await;

  std::process::exit(0);
}
pub struct CorePlugin {}

impl CorePlugin {
  pub fn new() -> Self {
    Self {}
  }
}

impl plugin::Plugin for CorePlugin {
  fn name(&self) -> &str {
    "core"
  }

  fn commands(&self) -> indexmap::IndexMap<String, crate::command::CommandMetadata> {
    let mut cmds = IndexMap::new();

    let id_cmd = CommandMetadata::new(
      Permission::USER,
      "Shows chat identifier".to_string(),
      ReplyRequirement::Optional,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_id(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    let help_cmd = CommandMetadata::new(
      Permission::USER,
      "Shows help information".to_string(),
      ReplyRequirement::None,
      vec![ArgMetadata::new(
        "command".to_string(),
        "Command name to get detailed info".to_string(),
        ArgRequirement::Optional,
      )],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_help(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    let shutdown_cmd = CommandMetadata::new(
      Permission::OWNER,
      "Shuting down the bot".to_string(),
      ReplyRequirement::None,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_shutdown(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    let uptime_cmd = CommandMetadata::new(
      Permission::USER,
      "Shows the uptime".to_string(),
      ReplyRequirement::None,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_uptime(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    cmds.insert("id".to_string(), id_cmd);
    cmds.insert("help".to_string(), help_cmd);
    cmds.insert("shutdown".to_string(), shutdown_cmd);
    cmds.insert("uptime".to_string(), uptime_cmd);

    cmds
  }

  fn update_handlers(&self) -> Vec<crate::handler::UpdateHandler> {
    Vec::new()
  }
}

pub async fn get_plugin() -> plugin::PluginBox {
  Box::new(CorePlugin::new())
}
