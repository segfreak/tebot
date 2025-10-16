use std::sync::Arc;
use std::sync::{Mutex, Weak};

use derivative::Derivative;
use indexmap::IndexMap;

use super::command;
use super::context;

pub type PluginBox = Box<dyn Plugin>;
pub type PluginMap = IndexMap<String, PluginBox>;

pub trait Plugin: Send + Sync {
  fn name(&self) -> &str;
  fn commands(&self) -> IndexMap<String, command::CommandMetadata>;
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct PluginCommandDispatcher {
  pub context: Weak<Mutex<context::Context>>,
  pub command_handlers: IndexMap<String, command::CommandMetadata>,

  #[derivative(Debug = "ignore")]
  pub plugins: IndexMap<String, PluginBox>,
}

impl PluginCommandDispatcher {
  pub fn new(context: Weak<Mutex<super::context::Context>>) -> Self {
    Self {
      context,
      command_handlers: IndexMap::new(),
      plugins: IndexMap::new(),
    }
  }

  pub fn new_arc_mutex(
    context: Weak<Mutex<super::context::Context>>,
  ) -> Arc<tokio::sync::Mutex<Self>> {
    Arc::new(tokio::sync::Mutex::new(Self::new(context)))
  }

  pub fn register_plugin(&mut self, plugin: PluginBox) {
    let plugin_name = plugin.name().to_string();
    log::debug!("[{}] registering", plugin_name);

    for (cmd_name, meta) in plugin.commands() {
      log::debug!(
        "[{}] command {} registered",
        plugin_name.clone(),
        cmd_name.clone()
      );

      self.command_handlers.insert(cmd_name, meta);
    }

    self.plugins.insert(plugin_name.clone(), plugin);
    log::debug!("[{}] successfully registered", plugin_name);
  }

  pub async fn handle_command(
    &self,
    bot: teloxide::Bot,
    msg: teloxide::prelude::Message,
    cmd: command::Command,
  ) {
    let user_id = match &msg.from {
      Some(user) => user.id,
      None => return,
    };

    if let Some(info) = self.command_handlers.get(&cmd.name) {
      if let Some(ctx) = self.context.upgrade() {
        let ctx = ctx.lock().unwrap();
        let cfg = ctx.cfg.lock().unwrap();
        let pm = ctx.perm_mgr.lock().unwrap();
        if pm.can(user_id, info.perm) {
          drop(cfg);
          (info.handler)(bot.clone(), msg.clone(), cmd, self.context.clone());
        }
      }
    }
  }

  pub async fn handle_message(
    &self,
    bot: teloxide::Bot,
    msg: teloxide::prelude::Message,
  ) -> Option<()> {
    let text = msg.text()?;
    let prefixes = if let Some(ctx) = self.context.upgrade() {
      let ctx = ctx.lock().unwrap();
      let cfg = ctx.cfg.lock().unwrap();
      cfg.get_prefixes()
    } else {
      log::error!("using context after it has been destroyed");
      return None;
    };

    let cmd = command::Command::with_prefixes(text, prefixes)?;
    self.handle_command(bot, msg, cmd).await;
    Some(())
  }
}
