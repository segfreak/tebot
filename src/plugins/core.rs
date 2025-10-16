use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

use indexmap::IndexMap;

use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{Message, ParseMode, UserId};
use teloxide::Bot;

use crate::command::{self, ArgMetadata, ArgRequirement, CommandMetadata, ReplyRequirement};
use crate::permissions::Permission;
use crate::{context, plugin};

pub async fn on_id(
  bot: Bot,
  msg: Message,
  _cmd: command::Command,
  _ctx: Weak<Mutex<context::Context>>,
) {
  let chat_id = msg.chat.id;
  let bot = bot.clone();

  let mut text = format!("<b>Chat ID</b>: <code>{}</code>\n", chat_id);

  if let Some(reply) = msg.reply_to_message() {
    let replied_user_id = reply.from.as_ref().map(|u| u.id).unwrap_or(UserId(0));
    text.push_str(&format!(
      "<b>User ID</b>: <code>{}</code>\n",
      replied_user_id
    ));
  }

  let _ = bot
    .send_message(chat_id, text)
    .parse_mode(ParseMode::Html)
    .await;
}

pub async fn on_legacy_help(
  bot: Bot,
  msg: Message,
  cmd: command::Command,
  _ctx: Weak<Mutex<context::Context>>,
) {
  let chat_id = msg.chat.id;

  let ctx = match _ctx.upgrade() {
    Some(ctx) => ctx,
    None => {
      log::error!("context disposed");
      let _ = bot.send_message(chat_id, "❌ Context disposed").await;
      return;
    }
  };

  let ctx_guard: tokio::sync::MutexGuard<'_, context::Context> = ctx.lock().await;

  let dp_guard = ctx_guard.dp.lock().await;

  let help_text = if let Some(command_name) = cmd.args.get(0) {
    if let Some(info) = dp_guard.command_handlers.get(command_name) {
      let args_desc: Vec<String> = info
        .args
        .iter()
        .map(|arg| {
          format!(
            "  • <b>{}</b> [<i>{:?}</i>] — {}",
            arg.name, arg.requirement, arg.description
          )
        })
        .collect();

      let prefix = ctx_guard.cfg.lock().await.get_prefixes()[0];
      format!(
                " ‣ Command <code>{}{}</code>\n ‣ Description: <b>{}</b>\n ‣ Arguments: \n{}\n ‣ Reply: <i>{:?}</i>",
                prefix,
                command_name,
                info.desc,
                args_desc.join("\n"),
                info.reply
            )
    } else {
      format!("Command <b>{}</b> not found", command_name)
    }
  } else {
    let prefix = ctx_guard.cfg.lock().await.get_prefixes()[0];
    dp_guard
      .command_handlers
      .iter()
      .map(|(name, info)| format!(" ‣ <code>{}{}</code> - {}", prefix, name, info.desc))
      .collect::<Vec<_>>()
      .join("\n")
  };

  let _ = bot
    .send_message(chat_id, help_text)
    .parse_mode(teloxide::types::ParseMode::Html)
    .await;
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

    let ping_cmd = CommandMetadata::new(
      Permission::USER,
      "Responds with pong".to_string(),
      ReplyRequirement::None,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          let _ = _bot
            .send_message(_msg.chat.id, "<b>pong</b>")
            .parse_mode(ParseMode::Html)
            .await;
        });
      }),
    );

    let help_cmd = CommandMetadata::new(
      Permission::USER,
      "".to_string(),
      ReplyRequirement::None,
      vec![ArgMetadata::new(
        "command".to_string(),
        "Shows help for this command".to_string(),
        ArgRequirement::Optional,
      )],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_legacy_help(_bot, _msg, _cmd, _ctx).await;
        });
      }),
    );

    cmds.insert("ping".to_string(), ping_cmd);
    cmds.insert("help".to_string(), help_cmd);

    cmds
  }

  fn update_handlers(&self) -> Vec<crate::handler::UpdateHandler> {
    Vec::new()
  }
}

pub fn get_plugin() -> plugin::PluginBox {
  Box::new(CorePlugin::new())
}
