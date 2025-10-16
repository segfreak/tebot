use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

use indexmap::IndexMap;

use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{Message, UserId};
use teloxide::Bot;

use crate::command::{self, ArgMetadata, ArgRequirement, CommandMetadata, ReplyRequirement};
use crate::permissions::Permission;
use crate::{context, parsers, plugin};

pub async fn on_id(
  bot: Bot,
  msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) {
  let chat_id = msg.chat.id;

  let mut text = String::new();

  text.push_str(&format!("‣ <b>Chat ID</b>: <code>{}</code>\n", chat_id));

  if let Some(reply) = msg.reply_to_message() {
    let replied_user_id = reply.from.as_ref().map(|u| u.id).unwrap_or(UserId(0));
    text.push_str(&format!(
      "‣ <b>User ID</b>: <code>{}</code>\n",
      replied_user_id
    ));
  }

  let _ = bot
    .send_message(chat_id, text)
    .parse_mode(teloxide::types::ParseMode::Html)
    .await;
}

pub async fn on_help(
  bot: Bot,
  msg: Message,
  cmd: command::Command,
  ctx_weak: Weak<tokio::sync::Mutex<context::Context>>,
) {
  let chat_id = msg.chat.id;

  let ctx = match ctx_weak.upgrade() {
    Some(ctx) => ctx,
    None => {
      log::error!("context disposed");
      let _ = bot.send_message(chat_id, "⨯ Context disposed").await;
      return;
    }
  };

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
            "‣ {} [{}] → {}",
            arg.name,
            format!("{:?}", arg.requirement),
            arg.description
          )
        })
        .collect();

      format!(
        "Command: <code>{}{}</code>\n\
                 Description: {}\n\
                 Arguments:\n{}\n\
                 Reply: <i>{:?}</i>",
        prefix,
        command_name,
        info.desc,
        args_desc.join("\n"),
        info.reply
      )
    } else {
      format!("⨯ Command <b>{}</b> not found", command_name)
    }
  } else {
    let mut sections = Vec::new();

    for (plugin_name, plugin) in &dp_guard.plugins {
      let commands_list: Vec<String> = plugin
        .commands()
        .iter()
        .map(|(name, info)| format!("• <code>{}{}</code> → {}", prefix, name, info.desc))
        .collect();

      let section = format!("{}:\n{}", plugin_name, commands_list.join("\n"));
      sections.push(section);
    }

    sections.join("\n\n")
  };

  let _ = bot
    .send_message(chat_id, help_text)
    .parse_mode(teloxide::types::ParseMode::Html)
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
      let _ = bot.send_message(chat_id, "⨯ Context disposed").await;
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

async fn handle_role_change(
  bot: Bot,
  msg: Message,
  cmd: command::Command,
  _ctx: Weak<Mutex<context::Context>>,
  add: bool,
) {
  let chat_id = msg.chat.id;
  let bot = bot.clone();
  let args = &cmd.args;

  let ctx = match _ctx.upgrade() {
    Some(ctx) => ctx,
    None => {
      log::error!("Context disposed");
      let _ = bot.send_message(chat_id, "⨯ Context disposed").await;
      return;
    }
  };

  let ctx_guard = ctx.lock().await;
  let pmgr_guard = ctx_guard.perm_mgr.lock().await;

  let (user_id, role_str) = if let Some(reply) = msg.reply_to_message() {
    (
      reply.from.as_ref().map(|u| u.id).unwrap_or(UserId(0)),
      args.get(0).map(|s| s.as_str()),
    )
  } else {
    if args.len() < 2 {
      let _ = bot
        .send_message(chat_id, "⨯ Usage: /role <user_id> <role>")
        .await;
      return;
    }
    let user_id = match parsers::parse_uid(&args[0]) {
      Ok(id) => id,
      Err(_) => {
        let _ = bot.send_message(chat_id, "⨯ Invalid user ID").await;
        return;
      }
    };
    (user_id, args.get(1).map(|s| s.as_str()))
  };

  let role = match role_str {
    Some(r) => match parsers::parse_permission(r) {
      Ok(r) => r,
      Err(_) => {
        let _ = bot
          .send_message(chat_id, format!("⨯ Unknown role: {}", r))
          .await;
        return;
      }
    },
    None => {
      let _ = bot.send_message(chat_id, "⨯ Role not specified").await;
      return;
    }
  };

  if add {
    pmgr_guard.set(user_id, role);
  } else {
    pmgr_guard.revoke(user_id, role);
  }

  let action = if add { "added to" } else { "removed from" };
  let msg_text = format!(
    "‣ Role <b>{:?}</b> successfully {} user <b>{}</b>",
    role, action, user_id
  );

  let _ = bot
    .send_message(chat_id, msg_text)
    .parse_mode(teloxide::types::ParseMode::Html)
    .await;
}

pub async fn on_addrole(
  bot: Bot,
  msg: Message,
  cmd: command::Command,
  ctx: Weak<Mutex<context::Context>>,
) {
  handle_role_change(bot, msg, cmd, ctx, true).await;
}

pub async fn on_remrole(
  bot: Bot,
  msg: Message,
  cmd: command::Command,
  ctx: Weak<Mutex<context::Context>>,
) {
  handle_role_change(bot, msg, cmd, ctx, false).await;
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
          on_id(_bot, _msg, _cmd, _ctx).await;
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
          on_help(_bot, _msg, _cmd, _ctx).await;
        });
      }),
    );

    let addrole_cmd = CommandMetadata::new(
      Permission::OWNER,
      "Add a role to a user".to_string(),
      ReplyRequirement::None,
      vec![
        ArgMetadata::new(
          "user_id".to_string(),
          "User Id to assign role".to_string(),
          ArgRequirement::OnlyWithoutReply,
        ),
        ArgMetadata::new(
          "role".to_string(),
          "Role to assign".to_string(),
          ArgRequirement::Required,
        ),
      ],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_addrole(_bot, _msg, _cmd, _ctx).await;
        });
      }),
    );

    let remrole_cmd = CommandMetadata::new(
      Permission::OWNER,
      "Remove a role from a user".to_string(),
      ReplyRequirement::None,
      vec![
        ArgMetadata::new(
          "user_id".to_string(),
          "User Id to remove role".to_string(),
          ArgRequirement::OnlyWithoutReply,
        ),
        ArgMetadata::new(
          "role".to_string(),
          "Role to remove".to_string(),
          ArgRequirement::Required,
        ),
      ],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_remrole(_bot, _msg, _cmd, _ctx).await;
        });
      }),
    );

    cmds.insert("help".to_string(), help_cmd);
    cmds.insert("id".to_string(), id_cmd);
    cmds.insert("addrole".to_string(), addrole_cmd);
    cmds.insert("remrole".to_string(), remrole_cmd);

    cmds
  }

  fn update_handlers(&self) -> Vec<crate::handler::UpdateHandler> {
    Vec::new()
  }
}

pub fn get_plugin() -> plugin::PluginBox {
  Box::new(CorePlugin::new())
}
