use std::sync::{Arc, Weak};

use indexmap::IndexMap;

use teloxide::payloads::*;
use teloxide::prelude::Requester;
use teloxide::types::{Message, UserId};
use teloxide::Bot;

use crate::bot::command::{self, ArgMetadata, ArgRequirement, CommandMetadata, ReplyRequirement};
use crate::permissions::types::Permission;

use crate::{
  bot::{context, handler, plugin},
  error,
  utils::{metadata, style},
};

async fn on_id(
  bot: Bot,
  msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let chat_id = msg.chat.id;

  let _style = style::get_style(_ctx.clone()).await;

  let mut text = String::new();

  text.push_str(&format!(
    "{} <b>Chat ID</b>: <code>{}</code>\n",
    _style.bullet(),
    chat_id
  ));

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
          error::Error::CommandNotFound(command_name.to_string()),
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

async fn on_version(
  _bot: Bot,
  _msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let _style = style::get_style(_ctx.clone()).await;
  let _msg_text = format!(
    "{} <b>Package</b>: <b>{}</b>\n{} <b>Git</b>: <b>{}</b>",
    _style.arrow(),
    metadata::Package::from_env()?,
    _style.arrow(),
    metadata::GitMetadata::from_env()?,
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

async fn on_ping(
  _bot: Bot,
  _msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let _style = style::get_style(_ctx.clone()).await;
  let _start = std::time::Instant::now();

  let _ping_msg = _bot
    .send_message(_msg.chat.id, format!("{} Pinging...", _style.arrow()))
    .parse_mode(teloxide::types::ParseMode::Html)
    .await?;

  let _latency = _start.elapsed();

  let _msg_text = format!(
    "{} <b>Pong!</b> Latency: <code>{}ms</code>",
    _style.ok(),
    _latency.as_millis()
  );

  let _ = _bot
    .edit_message_text(_msg.chat.id, _ping_msg.id, _msg_text)
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
    "core"
  }

  fn commands(&self) -> indexmap::IndexMap<String, command::CommandMetadata> {
    let mut cmds = IndexMap::new();

    let id_cmd = CommandMetadata::new(
      Permission::USER,
      "Get chat and user identifiers".to_string(),
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
      "Display available commands and their usage".to_string(),
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
      "Shutdown the bot process".to_string(),
      ReplyRequirement::None,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_shutdown(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    let version_cmd = CommandMetadata::new(
      Permission::USER,
      "Display all information".to_string(),
      ReplyRequirement::None,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_version(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    let ping_cmd = CommandMetadata::new(
      Permission::USER,
      "Check bot response time and latency".to_string(),
      ReplyRequirement::None,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_ping(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    cmds.insert("id".to_string(), id_cmd);
    cmds.insert("help".to_string(), help_cmd);
    cmds.insert("shutdown".to_string(), shutdown_cmd);
    cmds.insert("version".to_string(), version_cmd);
    cmds.insert("ping".to_string(), ping_cmd);

    cmds
  }

  fn update_handlers(&self) -> Vec<handler::UpdateHandler> {
    Vec::new()
  }
}

pub fn get_plugin() -> plugin::PluginBox {
  Box::new(Plugin::new())
}
