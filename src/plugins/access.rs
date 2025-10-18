use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

use indexmap::IndexMap;

use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{Message, UserId};
use teloxide::Bot;

use crate::bot::command::{self, ArgMetadata, ArgRequirement, CommandMetadata, ReplyRequirement};
use crate::permissions::types::Permission;
use crate::plugins::core::CoreError;

use crate::{
  bot::{context, handler, plugin},
  error,
  utils::{parsers, style},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionEvent {
  Grant,
  Revoke,
  Set,
  Reset,
}

async fn handle_perm_event(
  _bot: Bot,
  _msg: Message,
  _weak_ctx: Weak<Mutex<context::Context>>,
  _event: PermissionEvent,
  _perm: Option<Permission>,
  _user_id: Option<UserId>,
) -> anyhow::Result<()> {
  let _ctx = match _weak_ctx.upgrade() {
    Some(ctx) => ctx,
    None => {
      return Err(
        error::emit(
          Some(_bot.clone()),
          Some(_msg.clone()),
          error::Error::ContextDisposed,
        )
        .await,
      )
    }
  };

  let _style = style::get_style(_weak_ctx.clone()).await;

  let _ctx_guard = _ctx.lock().await;
  let _pm_guard = _ctx_guard.perm_mgr.lock().await;

  let _perm_needed: bool;

  let _user_id = match _user_id {
    Some(id) => id,
    None => {
      return Err(
        error::emit(
          Some(_bot.clone()),
          Some(_msg.clone()),
          CoreError::OptionNotSpecified("user_id".to_string()),
        )
        .await,
      );
    }
  };

  match _event {
    PermissionEvent::Grant => {
      _perm_needed = true;

      if let Some(_perm) = _perm {
        _pm_guard.grant(_user_id, _perm)?;
      } else {
        return Err(
          error::emit(
            Some(_bot.clone()),
            Some(_msg.clone()),
            CoreError::OptionNotSpecified("permission".to_string()),
          )
          .await,
        );
      }
    }
    PermissionEvent::Revoke => {
      _perm_needed = true;

      if let Some(_perm) = _perm {
        _pm_guard.revoke(_user_id, _perm)?;
      } else {
        return Err(
          error::emit(
            Some(_bot.clone()),
            Some(_msg.clone()),
            CoreError::OptionNotSpecified("permission".to_string()),
          )
          .await,
        );
      }
    }
    PermissionEvent::Set => {
      _perm_needed = true;

      if let Some(_perm) = _perm {
        _pm_guard.set(_user_id, _perm)?;
      } else {
        return Err(
          error::emit(
            Some(_bot.clone()),
            Some(_msg.clone()),
            CoreError::OptionNotSpecified("permission".to_string()),
          )
          .await,
        );
      }
    }
    PermissionEvent::Reset => {
      _perm_needed = false;

      _pm_guard.reset(_user_id)?;
    }
  }

  let _action = match _event {
    PermissionEvent::Grant => "granted",
    PermissionEvent::Revoke => "revoked",
    PermissionEvent::Set => "setted",
    PermissionEvent::Reset => "resetted",
  };

  let _msg_text = match _event {
    PermissionEvent::Grant | PermissionEvent::Revoke | PermissionEvent::Set => {
      let action_verb = match _event {
        PermissionEvent::Grant => "granted",
        PermissionEvent::Revoke => "revoked",
        PermissionEvent::Set => "set",
        _ => unreachable!(),
      };

      let perm_name = _perm
        .map(|p| format!("{:?}", p))
        .unwrap_or_else(|| "unknown".to_string());

      format!(
        "{} <b>Permission Update</b>\n\
      {} <b>User:</b> <code>{}</code>\n\
      {} <b>Permission:</b> <code>{}</code>\n\
      {} <b>Action:</b> {}\n",
        _style.info(),
        _style.info(),
        _user_id,
        _style.info(),
        perm_name,
        _style.info(),
        action_verb,
      )
    }

    PermissionEvent::Reset => format!(
      "{} <b>Permissions Reset</b>\n\
    {} <b>User:</b> <code>{}</code>\n\
    {} All permissions have been reset.\n",
      _style.info(),
      _user_id,
      _style.info(),
      _style.info(),
    ),
  };

  let _ = _bot
    .send_message(_msg.chat.id, _msg_text)
    .parse_mode(teloxide::types::ParseMode::Html)
    .await?;

  Ok(())
}

async fn parse_user_id_and_perm(
  _bot: &Bot,
  _msg: &Message,
  _cmd: &command::Command,
  _require_userid: bool,
  _require_perm: bool,
) -> anyhow::Result<(Option<UserId>, Option<Permission>)> {
  let _user_id = if let Some(reply) = &_msg.reply_to_message() {
    reply.from.as_ref().map(|u| u.id)
  } else if _cmd.args.is_empty() {
    if _require_userid {
      return Err(
        error::emit(
          Some(_bot.clone()),
          Some(_msg.clone()),
          CoreError::OptionNotSpecified("user_id".to_string()),
        )
        .await,
      );
    }
    None
  } else {
    match parsers::parse_uid(&_cmd.args[0]).await {
      Ok(uid) => Some(uid),
      Err(_) => {
        if _require_userid {
          return Err(
            error::emit(
              Some(_bot.clone()),
              Some(_msg.clone()),
              CoreError::InvalidOption("user_id".to_string()),
            )
            .await,
          );
        }
        None
      }
    }
  };

  let _perm = if _require_perm {
    let perm_str = if let Some(_reply) = &_msg.reply_to_message() {
      _cmd.args.get(0)
    } else {
      let index = if _require_userid { 1 } else { 0 };
      _cmd.args.get(index)
    };

    let perm_str = match perm_str {
      Some(p) => p,
      None => {
        return Err(
          error::emit(
            Some(_bot.clone()),
            Some(_msg.clone()),
            CoreError::OptionNotSpecified("perm".to_string()),
          )
          .await,
        );
      }
    };

    match parsers::parse_permission(perm_str).await {
      Ok(perm) => Some(perm),
      Err(_) => {
        return Err(
          error::emit(
            Some(_bot.clone()),
            Some(_msg.clone()),
            CoreError::UnknownOption(format!("perm {}", perm_str)),
          )
          .await,
        );
      }
    }
  } else {
    None
  };

  Ok((_user_id, _perm))
}

async fn on_grant(
  bot: Bot,
  msg: Message,
  cmd: command::Command,
  ctx: Weak<Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let (user_id, perm) = parse_user_id_and_perm(&bot, &msg, &cmd, true, true).await?;
  handle_perm_event(bot, msg, ctx, PermissionEvent::Grant, perm, user_id).await
}

async fn on_revoke(
  bot: Bot,
  msg: Message,
  cmd: command::Command,
  ctx: Weak<Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let (user_id, perm) = parse_user_id_and_perm(&bot, &msg, &cmd, true, true).await?;
  handle_perm_event(bot, msg, ctx, PermissionEvent::Revoke, perm, user_id).await
}

async fn on_set(
  bot: Bot,
  msg: Message,
  cmd: command::Command,
  ctx: Weak<Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let (user_id, perm) = parse_user_id_and_perm(&bot, &msg, &cmd, true, true).await?;
  handle_perm_event(bot, msg, ctx, PermissionEvent::Set, perm, user_id).await
}

async fn on_reset(
  bot: Bot,
  msg: Message,
  cmd: command::Command,
  ctx: Weak<Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let (user_id, _) = parse_user_id_and_perm(&bot, &msg, &cmd, true, false).await?;
  handle_perm_event(bot, msg, ctx, PermissionEvent::Reset, None, user_id).await
}

async fn on_show(
  _bot: Bot,
  _msg: Message,
  _cmd: command::Command,
  _weak_ctx: Weak<Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let (_user_id, _) = parse_user_id_and_perm(&_bot, &_msg, &_cmd, false, false).await?;

  let _ctx = match _weak_ctx.upgrade() {
    Some(ctx) => ctx,
    None => {
      return Err(
        error::emit(
          Some(_bot.clone()),
          Some(_msg.clone()),
          error::Error::ContextDisposed,
        )
        .await,
      )
    }
  };

  let _style = style::get_style(_weak_ctx.clone()).await;

  let _ctx_guard = _ctx.lock().await;
  let _pm_guard = _ctx_guard.perm_mgr.lock().await;

  let _pm_map = _pm_guard.snapshot()?;

  if let Some(_uid) = _user_id {
    match _pm_map.get(&_uid) {
      Some(perm) => {
        let text = format!(
          "{} <b>User ID:</b> <code>{}</code>\n{} <b>Permission:</b> <code>{:?}</code>",
          _style.bullet(),
          _uid.0,
          _style.info(),
          perm
        );

        _bot
          .send_message(_msg.chat.id, text)
          .parse_mode(teloxide::types::ParseMode::Html)
          .await?;
      }
      None => {
        return Err(
          error::emit(
            Some(_bot.clone()),
            Some(_msg.clone()),
            CoreError::NotFound("permissions".to_string()),
          )
          .await,
        )
      }
    }
    return Ok(());
  }

  if _pm_map.is_empty() {
    return Err(
      error::emit(
        Some(_bot.clone()),
        Some(_msg.clone()),
        CoreError::IsEmpty("permission map".to_string()),
      )
      .await,
    );
  }

  let mut text = format!("{} <b>Current Permission Map:</b>\n\n", _style.bullet());
  for (uid, perm) in _pm_map {
    text.push_str(&format!(
      "{} User ID: <code>{}</code>\n  └─ Permission: <b>{:?}</b>\n",
      _style.info(),
      uid.0,
      perm
    ));
  }

  _bot
    .send_message(_msg.chat.id, text)
    .parse_mode(teloxide::types::ParseMode::Html)
    .await?;

  Ok(())
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

  fn commands(&self) -> indexmap::IndexMap<String, command::CommandMetadata> {
    let mut cmds = IndexMap::new();

    let grant_cmd = CommandMetadata::new(
      Permission::OWNER,
      "Grants a permission to a user".to_string(),
      ReplyRequirement::None,
      vec![
        ArgMetadata::new(
          "user_id".to_string(),
          "User Id to grant perm".to_string(),
          ArgRequirement::OnlyWithoutReply,
        ),
        ArgMetadata::new(
          "perm".to_string(),
          "Permission to grant".to_string(),
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
      "Rovokes a permission from a user".to_string(),
      ReplyRequirement::None,
      vec![
        ArgMetadata::new(
          "user_id".to_string(),
          "User Id to revoke permission".to_string(),
          ArgRequirement::OnlyWithoutReply,
        ),
        ArgMetadata::new(
          "perm".to_string(),
          "Permission to revoke".to_string(),
          ArgRequirement::Required,
        ),
      ],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_revoke(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    let set_cmd = CommandMetadata::new(
      Permission::OWNER,
      "Sets a permission to a user".to_string(),
      ReplyRequirement::None,
      vec![
        ArgMetadata::new(
          "user_id".to_string(),
          "User Id to set permission".to_string(),
          ArgRequirement::OnlyWithoutReply,
        ),
        ArgMetadata::new(
          "perm".to_string(),
          "Permission to set".to_string(),
          ArgRequirement::Required,
        ),
      ],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_set(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    let reset_cmd = CommandMetadata::new(
      Permission::OWNER,
      "Resets a user permissions".to_string(),
      ReplyRequirement::None,
      vec![ArgMetadata::new(
        "user_id".to_string(),
        "User Id to reset perm".to_string(),
        ArgRequirement::OnlyWithoutReply,
      )],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_reset(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    let show_cmd = CommandMetadata::new(
      Permission::OWNER,
      "Show permissions".to_string(),
      ReplyRequirement::None,
      vec![ArgMetadata::new(
        "user_id".to_string(),
        "User Id to show permissions".to_string(),
        ArgRequirement::OnlyWithoutReply,
      )],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_show(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    cmds.insert("pmgrant".to_string(), grant_cmd);
    cmds.insert("pmrevoke".to_string(), revoke_cmd);
    cmds.insert("pmset".to_string(), set_cmd);
    cmds.insert("pmreset".to_string(), reset_cmd);
    cmds.insert("pmshow".to_string(), show_cmd);

    cmds
  }

  fn update_handlers(&self) -> Vec<handler::UpdateHandler> {
    Vec::new()
  }
}

pub async fn get_plugin() -> plugin::PluginBox {
  Box::new(AccessPlugin::new())
}
