use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

use derivative::Derivative;
use indexmap::IndexMap;

use super::command;
use super::context;
use super::handler;
use super::plugin;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Dispatcher {
  pub context: Weak<Mutex<context::Context>>,

  pub command_handlers: IndexMap<String, command::CommandMetadata>,

  #[derivative(Debug = "ignore")]
  pub update_handlers: Vec<handler::UpdateHandler>,

  #[derivative(Debug = "ignore")]
  pub plugins: IndexMap<String, plugin::PluginBox>,
}

impl Dispatcher {
  pub fn new(context: Weak<Mutex<super::context::Context>>) -> Self {
    Self {
      context,
      command_handlers: IndexMap::new(),
      update_handlers: Vec::new(),
      plugins: IndexMap::new(),
    }
  }

  pub fn new_arc_mutex(
    context: Weak<Mutex<super::context::Context>>,
  ) -> Arc<tokio::sync::Mutex<Self>> {
    Arc::new(tokio::sync::Mutex::new(Self::new(context)))
  }

  pub async fn register_plugin(&mut self, plugin: plugin::PluginBox) {
    let plugin_name = plugin.name().to_string();

    for update in plugin.update_handlers() {
      self.update_handlers.push(update);
    }

    for (cmd_name, meta) in plugin.commands() {
      log::debug!(
        "registering '{}' ({:?}) from plugin '{}'",
        cmd_name,
        meta.perm,
        plugin_name
      );

      self.command_handlers.insert(cmd_name, meta);
    }

    self.plugins.insert(plugin_name.clone(), plugin);
  }

  pub async fn handle_command(
    &self,
    bot: teloxide::Bot,
    msg: teloxide::prelude::Message,
    cmd: command::Command,
  ) -> anyhow::Result<()> {
    let user_id = match &msg.from {
      Some(user) => user.id,
      None => {
        log::trace!("command {} ignored: message has no sender", cmd.name);
        return Ok(());
      }
    };

    if let Some(info) = self.command_handlers.get(&cmd.name) {
      if let Some(ctx) = self.context.upgrade() {
        let ctx = ctx.lock().await;
        let cfg = ctx.cfg.lock().await;
        let pm = ctx.perm_mgr.lock().await;

        if pm.can(user_id, info.perm)? {
          drop(cfg);
          log::trace!("executing command {} for user {}", cmd.name, user_id);
          (info.handler)(bot.clone(), msg.clone(), cmd, self.context.clone());
        } else {
          log::trace!(
            "user {} does not have permission for command {}",
            user_id,
            cmd.name
          );
        }
      } else {
        log::warn!("cannot execute command {}: context dropped", cmd.name);
      }
    } else {
      log::trace!(
        "unknown command {} received from user {}",
        cmd.name,
        user_id
      );
    }

    Ok(())
  }

  pub async fn handle_message(
    &self,
    bot: teloxide::Bot,
    msg: teloxide::prelude::Message,
  ) -> anyhow::Result<()> {
    let text = msg
      .text()
      .ok_or_else(|| anyhow::anyhow!("message has no text"))?;

    let prefixes = if let Some(ctx) = self.context.upgrade() {
      let ctx = ctx.lock().await;
      let cfg = ctx.cfg.lock().await;
      cfg.get_prefixes()
    } else {
      anyhow::bail!("cannot handle message: context already destroyed");
    };

    let cmd = command::Command::with_prefixes(text, prefixes)
      .ok_or_else(|| anyhow::anyhow!("message does not match any command"))?;

    log::trace!(
      "handling message as command {} from user {:?}",
      cmd.name,
      msg.from.as_ref().map(|u| u.id)
    );

    self.handle_command(bot, msg, cmd).await?;

    Ok(())
  }

  pub async fn handle_update(
    &self,
    bot: teloxide::Bot,
    update: teloxide::prelude::Update,
  ) -> anyhow::Result<()> {
    for handler in &self.update_handlers {
      if self.context.upgrade().is_some() {
        (handler)(bot.clone(), update.clone(), self.context.clone()).await;
      } else {
        anyhow::bail!("cannot handle message: context already destroyed");
      }
    }

    if let teloxide::types::UpdateKind::Message(msg) = update.kind {
      log::trace!("update contains message, handling message");
      self.handle_message(bot.clone(), msg).await?;
    }

    Ok(())
  }
}
