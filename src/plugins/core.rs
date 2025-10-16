use std::sync::Arc;

use indexmap::IndexMap;

use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::ParseMode;

use crate::command::{CommandMetadata, ReplyRequirement};
use crate::permissions::Permission;
use crate::plugin;

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

    cmds.insert("ping".to_string(), ping_cmd);
    cmds
  }
}

pub fn get_plugin() -> plugin::PluginBox {
  Box::new(CorePlugin::new())
}
