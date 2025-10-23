use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};

use indexmap::IndexMap;

use teloxide::net::Download;
use teloxide::payloads::*;
use teloxide::prelude::*;
use teloxide::types::*;

use teloxide::Bot;
use tokio::io::AsyncWriteExt;

use crate::bot::command::{self, CommandMetadata, ReplyRequirement};
use crate::permissions::types::Permission;

use crate::utils::{self, dirs};
use crate::{
  bot::{context, handler, plugin},
  error,
  utils::style,
};

pub async fn download_file(
  bot: &Bot,
  file_id: FileId,
  path: &PathBuf,
) -> anyhow::Result<()> {
  let file = bot.get_file(file_id).await?;
  let mut out_file = tokio::fs::File::create(path).await?;
  bot.download_file(&file.path, &mut out_file).await?;
  out_file.flush().await?;
  Ok(())
}

async fn on_load(
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

  let _filename = &_doc.file_name.clone().unwrap_or("__unnammed__".to_string());
  let _path = dirs::join_dir(Some(_filename)).await;

  let mut _load_msg = _bot
    .send_message(
      _msg.chat.id,
      format!("{} <b>Loading file...</b>", _style.bullet()),
    )
    .parse_mode(ParseMode::Html)
    .await?;

  super::storage::download_file(&_bot, _doc.file.id.clone(), &_path).await?;

  let _ = _bot
    .edit_message_text(
      _msg.chat.id,
      _load_msg.id,
      format!(
        "{} <b>File downloaded at</b> <code>{}</code>",
        _style.bullet(),
        _path.to_string_lossy()
      ),
    )
    .parse_mode(ParseMode::Html)
    .await?;

  Ok(())
}

async fn on_drop(
  _bot: Bot,
  _msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let _style = style::get_style(_ctx.clone()).await;

  if let Some(_file) = _cmd.args.get(0) {
    let _path = dirs::join_dir(Some(_file)).await;

    if !_path.exists() {
      return Err(
        error::emit(
          Some(_bot.clone()),
          Some(_msg.clone()),
          error::Error::NotFound("file".to_string()),
        )
        .await,
      );
    }

    let mut _drop_msg = _bot
      .send_message(
        _msg.chat.id,
        format!("{} <b>Dropping file...</b>", _style.bullet()),
      )
      .parse_mode(ParseMode::Html)
      .await?;

    _bot
      .send_document(_msg.chat.id, InputFile::file(_path.clone()))
      .caption(format!("{} <b>Success</b>", _style.bullet(),))
      .parse_mode(ParseMode::Html)
      .await?;

    _bot.delete_message(_drop_msg.chat.id, _drop_msg.id).await?;
  }

  Ok(())
}

#[async_recursion::async_recursion]
async fn format_tree(
  style: Arc<dyn utils::style::DynStyle>,
  dir: &Path,
  depth: usize,
) -> std::io::Result<String> {
  let mut result = String::new();
  let mut entries = tokio::fs::read_dir(dir).await?;

  while let Some(entry) = entries.next_entry().await? {
    let file_type = entry.file_type().await?;
    let name = entry.file_name().to_string_lossy().to_string();
    let is_dir = file_type.is_dir();

    let icon = if is_dir { style.arrow() } else { style.info() };
    let indent = "  ".repeat(depth);
    result.push_str(&format!("{}{} <code>{}</code>\n", indent, icon, name));

    if is_dir {
      let sub_path = entry.path();
      let sub_tree = format_tree(style.clone(), &sub_path, depth + 1).await?;
      result.push_str(&sub_tree);
    }
  }

  Ok(result)
}

async fn on_list(
  _bot: Bot,
  _msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let _style = style::get_style(_ctx.clone()).await;

  let _root = dirs::join_dir(None).await;
  let _tree = format_tree(_style.clone(), &_root, 0).await?;

  let _msg_text = format!("{} File tree:\n{}", _style.bullet(), _tree);

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
    "storage"
  }

  fn commands(&self) -> indexmap::IndexMap<String, command::CommandMetadata> {
    let mut cmds = IndexMap::new();

    let load_cmd = CommandMetadata::new(
      Permission::ADMIN,
      "Loads file".to_string(),
      ReplyRequirement::OnlyWithoutDocument,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_load(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    let drop_cmd = CommandMetadata::new(
      Permission::ADMIN,
      "Drops file".to_string(),
      ReplyRequirement::None,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_drop(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    let list_cmd = CommandMetadata::new(
      Permission::ADMIN,
      "Lists files".to_string(),
      ReplyRequirement::None,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_list(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    cmds.insert("stload".to_string(), load_cmd);
    cmds.insert("stdrop".to_string(), drop_cmd);
    cmds.insert("stlist".to_string(), list_cmd);

    cmds
  }

  fn update_handlers(&self) -> Vec<handler::UpdateHandler> {
    Vec::new()
  }
}

pub fn get_plugin() -> plugin::PluginBox {
  Box::new(Plugin::new())
}
