use std::sync::{Arc, Weak};

use indexmap::IndexMap;

use teloxide::payloads::*;
use teloxide::prelude::*;
use teloxide::types::*;
use teloxide::Bot;

use crate::bot::command::{self, ArgMetadata, CommandMetadata, ReplyRequirement};
use crate::error;
use crate::permissions::types::Permission;
use crate::utils;
use crate::utils::dirs;

use crate::{
  bot::{context, handler, plugin},
  utils::style,
};

async fn on_extract(
  _bot: Bot,
  _msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let _style = style::get_style(_ctx.clone()).await;

  let _file = if let Some(_reply) = _msg.reply_to_message() {
    _reply.document()
  } else {
    None
  };

  let _file = match _file {
    Some(f) => f,
    None => {
      return Err(
        error::emit(
          Some(_bot.clone()),
          Some(_msg.clone()),
          error::Error::OptionNotSpecified("reply".to_string()),
        )
        .await,
      )
    }
  };

  let _filename = &_file
    .file_name
    .clone()
    .unwrap_or("unnammed.dll".to_string());

  let mut _path = std::env::temp_dir();
  _path.push(_filename);

  let mut _extract_msg = _bot
    .send_message(
      _msg.chat.id,
      format!("{} <b>extracting signature...</b>", _style.bullet()),
    )
    .parse_mode(ParseMode::Html)
    .await?;

  super::storage::download_file(&_bot, _file.file.id.clone(), &_path).await?;

  let _sig = tokio::task::spawn_blocking({
    let _path = _path.clone();
    move || sigthief::extract_signature(&_path)
  })
  .await?;

  match _sig {
    Ok(_sig) => {
      let _dir = dirs::plugin_data_dir(get_plugin()).await?;
      tokio::fs::create_dir_all(std::path::Path::new(&_dir)).await?;

      let mut _path = _dir.clone();
      _path.push(_filename);
      _path = _path.with_extension("sig");

      sigthief::save_signature(&_sig, &_path)?;

      _bot
        .edit_message_text(
          _extract_msg.chat.id,
          _extract_msg.id,
          format!(
            "{} <b>signature extracted to</b>: <code>{}</code>",
            _style.bullet(),
            _path.to_string_lossy()
          ),
        )
        .parse_mode(ParseMode::Html)
        .await?;
    }
    Err(e) => {
      return Err(error::emit(Some(_bot.clone()), Some(_msg.clone()), e).await);
    }
  }

  Ok(())
}

async fn on_apply(
  _bot: Bot,
  _msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let _style = style::get_style(_ctx.clone()).await;

  let _doc = if let Some((_doc, _source)) = utils::etc::get_document(&_msg) {
    _doc
  } else {
    return Err(
      error::emit(
        Some(_bot.clone()),
        Some(_msg.clone()),
        error::Error::NotFound("document".to_string()),
      )
      .await,
    );
  };

  let _filename = &_doc
    .file_name
    .clone()
    .unwrap_or("unnammed.signed.exe".to_string());

  let mut _path = std::env::temp_dir();
  _path.push(_filename);

  let mut _apply_msg = _bot
    .send_message(
      _msg.chat.id,
      format!("{} <b>applying signature...</b>", _style.bullet()),
    )
    .parse_mode(ParseMode::Html)
    .await?;

  super::storage::download_file(&_bot, _doc.file.id.clone(), &_path).await?;

  if let Some(_signature) = _cmd.args.get(0) {
    let _result = tokio::task::spawn_blocking({
      let _path = _path.clone();
      let _signature = _signature.clone();
      move || {
        sigthief::load_signature(&_signature)
          .and_then(|_sig| sigthief::apply_signature(&_path, &_sig))
      }
    })
    .await?;

    match _result {
      Ok(_) => {
        _bot.delete_message(_msg.chat.id, _apply_msg.id).await?;
        _bot
          .send_document(_msg.chat.id, InputFile::file(_path.clone()))
          .caption(format!("{} <b>signature applied</b>", _style.bullet(),))
          .parse_mode(ParseMode::Html)
          .await?;

        tokio::fs::remove_file(_path.clone()).await?;
      }
      Err(e) => {
        return Err(error::emit(Some(_bot.clone()), Some(_msg.clone()), e).await);
      }
    }
  } else {
    return Err(
      error::emit(
        Some(_bot.clone()),
        Some(_msg.clone()),
        error::Error::OptionNotSpecified("signature".to_string()),
      )
      .await,
    );
  }

  Ok(())
}

async fn on_list(
  _bot: Bot,
  _msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let _style = style::get_style(_ctx.clone()).await;

  let mut _msg_text = format!("{} Digital signatures list:\n", _style.bullet());

  let _dir = dirs::plugin_data_dir(get_plugin()).await?;

  let mut _entries = tokio::fs::read_dir(_dir).await?;
  while let Some(_entry) = _entries.next_entry().await? {
    let _path = _entry.path();
    _msg_text.push_str(&format!("{} {}\n", _style.info(), _path.to_string_lossy()));
  }

  let _ = _bot
    .send_message(_msg.chat.id, _msg_text)
    .parse_mode(ParseMode::Html)
    .await?;

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
    "sigthief"
  }

  fn commands(&self) -> indexmap::IndexMap<String, command::CommandMetadata> {
    let mut cmds = IndexMap::new();

    let extract_cmd = CommandMetadata::new(
      Permission::ADMIN,
      "Extracts digital signature".to_string(),
      ReplyRequirement::Required,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_extract(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    let apply_cmd = CommandMetadata::new(
      Permission::ADMIN,
      "Apply a extracted digital signature".to_string(),
      ReplyRequirement::Required,
      vec![ArgMetadata::new(
        "signature".to_string(),
        "The name of the signature file to apply".to_string(),
        command::ArgRequirement::Required,
      )],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_apply(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    let list_cmd = CommandMetadata::new(
      Permission::ADMIN,
      "List all saved digital signatures".to_string(),
      ReplyRequirement::None,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_list(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    cmds.insert("sigextract".to_string(), extract_cmd);
    cmds.insert("sigapply".to_string(), apply_cmd);
    cmds.insert("siglist".to_string(), list_cmd);

    cmds
  }

  fn update_handlers(&self) -> Vec<handler::UpdateHandler> {
    Vec::new()
  }
}

pub fn get_plugin() -> plugin::PluginBox {
  Box::new(Plugin::new())
}
