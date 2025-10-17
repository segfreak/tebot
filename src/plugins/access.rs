use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

use indexmap::IndexMap;

use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{Message, UserId};
use teloxide::Bot;

use crate::plugins::core::CoreError;

use crate::command::{self, ArgMetadata, ArgRequirement, CommandMetadata, ReplyRequirement};
use crate::permissions::Permission;
use crate::{context, error, parsers, plugin, style};

async fn handle_role_change(
  bot: Bot,
  msg: Message,
  cmd: command::Command,
  _ctx: Weak<Mutex<context::Context>>,
  add: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let chat_id = msg.chat.id;
  let bot = bot.clone();
  let args = &cmd.args;

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
      );
    }
  };

  let style = style::get_style(_ctx.clone()).await;

  let ctx_guard = ctx.lock().await;
  let pmgr_guard = ctx_guard.perm_mgr.lock().await;

  let (user_id, role_str) = if let Some(reply) = msg.reply_to_message() {
    (
      reply.from.as_ref().map(|u| u.id).unwrap_or(UserId(0)),
      args.get(0).map(|s| s.as_str()),
    )
  } else {
    if args.len() < 2 {
      return Err(
        error::emit(
          Some(bot.clone()),
          Some(msg.clone()),
          CoreError::InvalidCommandUsage("/role <user_id> <role>".to_string()),
        )
        .await,
      );
    }
    let user_id = match parsers::parse_uid(&args[0]) {
      Ok(id) => id,
      Err(_) => {
        return Err(
          error::emit(
            Some(bot.clone()),
            Some(msg.clone()),
            CoreError::InvalidOption("user_id".to_string()),
          )
          .await,
        );
      }
    };
    (user_id, args.get(1).map(|s| s.as_str()))
  };

  let role = match role_str {
    Some(r) => match parsers::parse_permission(r) {
      Ok(r) => r,
      Err(_) => {
        return Err(
          error::emit(
            Some(bot.clone()),
            Some(msg.clone()),
            CoreError::UnknownOption(format!("role {}", r)),
          )
          .await,
        );
      }
    },
    None => {
      return Err(
        error::emit(
          Some(bot.clone()),
          Some(msg.clone()),
          CoreError::OptionNotSpecified("role".to_string()),
        )
        .await,
      );
    }
  };

  if add {
    pmgr_guard.grant(user_id, role);
  } else {
    pmgr_guard.revoke(user_id, role);
  }

  let action = if add { "added to" } else { "removed from" };
  let msg_text = format!(
    "{} Role <b>{:?}</b> successfully {} user <b>{}</b>",
    style.info(),
    role,
    action,
    user_id
  );

  let _ = bot
    .send_message(chat_id, msg_text)
    .parse_mode(teloxide::types::ParseMode::Html)
    .await;

  Ok(())
}

pub async fn on_grant(
  bot: Bot,
  msg: Message,
  cmd: command::Command,
  ctx: Weak<Mutex<context::Context>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  handle_role_change(bot, msg, cmd, ctx, true).await
}

pub async fn on_revoke(
  bot: Bot,
  msg: Message,
  cmd: command::Command,
  ctx: Weak<Mutex<context::Context>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  handle_role_change(bot, msg, cmd, ctx, false).await
}

pub struct AccessPlugin {}

impl AccessPlugin {
  pub fn new() -> Self {
    Self {}
  }
}

impl plugin::Plugin for AccessPlugin {
  fn name(&self) -> &str {
    "access"
  }

  fn commands(&self) -> indexmap::IndexMap<String, crate::command::CommandMetadata> {
    let mut cmds = IndexMap::new();

    let grant_cmd = CommandMetadata::new(
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
          on_grant(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    let revoke_cmd = CommandMetadata::new(
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
          on_revoke(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    cmds.insert("grant".to_string(), grant_cmd);
    cmds.insert("revoke".to_string(), revoke_cmd);

    cmds
  }

  fn update_handlers(&self) -> Vec<crate::handler::UpdateHandler> {
    Vec::new()
  }
}

pub fn get_plugin() -> plugin::PluginBox {
  Box::new(AccessPlugin::new())
}
