use std::sync::{Arc, Weak};
use sysinfo::System;

use indexmap::IndexMap;

use teloxide::payloads::*;
use teloxide::prelude::Requester;
use teloxide::types::Message;
use teloxide::Bot;

use crate::bot::command::{self, CommandMetadata, ReplyRequirement};
use crate::permissions::types::Permission;

use crate::{
  bot::{context, handler, plugin},
  utils::style,
};

async fn on_sysinfo(
  _bot: Bot,
  _msg: Message,
  _cmd: command::Command,
  _ctx: Weak<tokio::sync::Mutex<context::Context>>,
) -> anyhow::Result<()> {
  let _style = style::get_style(_ctx.clone()).await;

  let mut _sys = System::new_all();
  _sys.refresh_all();

  let _cpu_count = _sys.cpus().len();
  let _cpu_brand = _sys
    .cpus()
    .first()
    .map(|cpu| cpu.brand())
    .unwrap_or("Unknown");
  let _cpu_usage = _sys.global_cpu_usage();

  let _total_memory = _sys.total_memory() / 1024 / 1024;
  let _used_memory = _sys.used_memory() / 1024 / 1024;
  let _memory_usage = (_used_memory as f64 / _total_memory as f64) * 100.0;

  let _pid = sysinfo::get_current_pid().ok();
  let _process_memory = _pid
    .and_then(|p| _sys.process(p))
    .map(|proc| proc.memory() / 1024 / 1024)
    .unwrap_or(0);

  let _os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
  let _os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
  let _kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());

  let _msg_text = format!(
    "{} <b>System Information</b>\n\n\
    {} <b>OS</b>: {} {}\n\
    {} <b>Kernel</b>: {}\n\n\
    {} <b>CPU</b>: {}\n\
    {} <b>Cores</b>: {}\n\
    {} <b>CPU Usage</b>: <code>{:.1}%</code>\n\n\
    {} <b>Total RAM</b>: <code>{} MB</code>\n\
    {} <b>Used RAM</b>: <code>{} MB</code> (<code>{:.1}%</code>)\n\n\
    {} <b>Bot Process RAM</b>: <code>{} MB</code>",
    _style.ok(),
    _style.bullet(),
    _os_name,
    _os_version,
    _style.bullet(),
    _kernel_version,
    _style.bullet(),
    _cpu_brand,
    _style.bullet(),
    _cpu_count,
    _style.bullet(),
    _cpu_usage,
    _style.bullet(),
    _total_memory,
    _style.bullet(),
    _used_memory,
    _memory_usage,
    _style.bullet(),
    _process_memory
  );

  let _ = _bot
    .send_message(_msg.chat.id, _msg_text)
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
    "system"
  }

  fn commands(&self) -> indexmap::IndexMap<String, command::CommandMetadata> {
    let mut cmds = IndexMap::new();

    let sysinfo_cmd = CommandMetadata::new(
      Permission::ADMIN,
      "Display system resources and host information".to_string(),
      ReplyRequirement::None,
      vec![],
      Arc::new(|_bot, _msg, _cmd, _ctx| {
        tokio::spawn(async move {
          on_sysinfo(_bot, _msg, _cmd, _ctx).await.unwrap_or(());
        });
      }),
    );

    cmds.insert("sysinfo".to_string(), sysinfo_cmd);

    cmds
  }

  fn update_handlers(&self) -> Vec<handler::UpdateHandler> {
    Vec::new()
  }
}

pub fn get_plugin() -> plugin::PluginBox {
  Box::new(Plugin::new())
}
