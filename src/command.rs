use std::sync::Weak;
use std::sync::{Arc, Mutex};

use derivative::Derivative;
use indexmap::IndexMap;

use super::context;
use super::permissions;

pub type CommandHandler = Arc<
  dyn Fn(teloxide::Bot, teloxide::prelude::Message, Command, Weak<Mutex<context::Context>>)
    + Send
    + Sync,
>;

pub struct Command {
  pub prefix: char,
  pub name: String,
  pub args: Vec<String>,
}

impl Command {
  pub fn with_prefix(s: &str, prefix: char) -> Option<Self> {
    let mut chars = s.chars().peekable();
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    if chars.peek() != Some(&prefix) {
      return None;
    }
    chars.next();

    while let Some(&c) = chars.peek() {
      match c {
        '"' => {
          in_quotes = !in_quotes;
          chars.next();
        }
        ' ' if !in_quotes => {
          if !current.is_empty() {
            parts.push(current.clone());
            current.clear();
          }
          chars.next();
        }
        _ => {
          current.push(c);
          chars.next();
        }
      }
    }

    if !current.is_empty() {
      parts.push(current);
    }

    let name = parts.get(0)?.clone();
    let args = parts.into_iter().skip(1).collect();

    Some(Self {
      prefix: prefix,
      name: name,
      args: args,
    })
  }

  pub fn with_prefixes<T>(s: &str, allowed: T) -> Option<Self>
  where
    T: AsRef<[char]>,
  {
    let mut chars = s.chars();
    let first = chars.next()?;

    if allowed.as_ref().contains(&first) {
      Self::with_prefix(s, first)
    } else {
      None
    }
  }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ArgRequirement {
  Optional,
  OnlyWithReply,
  OnlyWithoutReply,
  Required,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ReplyRequirement {
  None,
  Optional,
  Required,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct ArgMetadata {
  pub name: String,
  pub description: String,
  pub requirement: ArgRequirement,
}

impl ArgMetadata {
  pub fn new(name: String, description: String, requirement: ArgRequirement) -> Self {
    Self {
      name: name,
      description: description,
      requirement: requirement,
    }
  }
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct CommandMetadata {
  pub perm: permissions::Permission,
  pub desc: String,

  pub reply: ReplyRequirement,
  pub args: Vec<ArgMetadata>,

  #[derivative(Debug = "ignore")]
  pub handler: CommandHandler,
}

impl CommandMetadata {
  pub fn new(
    perm: permissions::Permission,
    desc: String,
    reply: ReplyRequirement,
    args: Vec<ArgMetadata>,
    handler: CommandHandler,
  ) -> Self {
    Self {
      perm: perm,
      desc: desc,
      reply: reply,
      args: args,
      handler: handler,
    }
  }
}

#[derive(Clone, Debug)]
pub struct CommandDispatcher {
  pub context: Weak<Mutex<context::Context>>,
  pub command_handlers: IndexMap<String, CommandMetadata>,
}

impl CommandDispatcher {
  pub fn new(context: Weak<Mutex<context::Context>>) -> Self {
    Self {
      context: context.clone(),
      command_handlers: IndexMap::new(),
    }
  }

  pub fn new_arc_mutex(context: Weak<Mutex<context::Context>>) -> Arc<Mutex<Self>> {
    Arc::new(Mutex::new(Self::new(context)))
  }

  pub fn register_command_handler(&mut self, name: &str, meta: CommandMetadata) {
    self.command_handlers.insert(name.to_string(), meta);
  }

  pub async fn handle_command(
    &self,
    bot: teloxide::Bot,
    msg: teloxide::prelude::Message,
    cmd: Command,
  ) {
    let user_id = match &msg.from {
      Some(user) => user.id,
      None => return,
    };

    if let Some(info) = self.command_handlers.get(&cmd.name) {
      {
        if let Some(ctx) = self.context.upgrade() {
          let ctx = ctx.lock().unwrap();
          let cfg = ctx.cfg.lock().unwrap();
          let pm = ctx.perm_mgr.lock().unwrap();
          if pm.has(user_id, info.perm) {
            drop(cfg);
            (info.handler)(bot.clone(), msg.clone(), cmd, self.context.clone());
          }
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

    let cmd = Command::with_prefixes(text, prefixes)?;
    self.handle_command(bot, msg, cmd).await;
    Some(())
  }
}
