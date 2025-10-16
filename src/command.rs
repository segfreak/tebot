use derivative::Derivative;

use super::handler;
use super::permissions;

pub struct Command {
  pub prefix: char,
  pub name: String,
  pub args: Vec<String>,
}

impl Command {
  pub fn with_prefix(s: &str, prefix: char) -> Option<Self> {
    log::trace!(
      "parsing command from string '{}' with prefix '{}'",
      s,
      prefix
    );
    let mut chars = s.chars().peekable();
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    if chars.peek() != Some(&prefix) {
      log::trace!("string '{}' does not start with prefix '{}'", s, prefix);
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

    let name = match parts.get(0) {
      Some(n) => n.clone(),
      None => {
        log::debug!("no command name found in string '{}'", s);
        return None;
      }
    };
    let args = parts.into_iter().skip(1).collect();

    log::trace!("parsed command '{}' with args {:?}", name, args);

    Some(Self { prefix, name, args })
  }

  pub fn with_prefixes<T>(s: &str, allowed: T) -> Option<Self>
  where
    T: AsRef<[char]>,
  {
    let mut chars = s.chars();
    let first = chars.next()?;
    if allowed.as_ref().contains(&first) {
      log::trace!("string '{}' matches allowed prefixes, using '{}'", s, first);
      Self::with_prefix(s, first)
    } else {
      log::trace!("string '{}' does not match any allowed prefixes", s);
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
    log::trace!(
      "creating arg metadata: name='{}', description='{}', requirement={:?}",
      name,
      description,
      requirement
    );
    Self {
      name,
      description,
      requirement,
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
  pub handler: handler::CommandHandler,
}

impl CommandMetadata {
  pub fn new(
    perm: permissions::Permission,
    desc: String,
    reply: ReplyRequirement,
    args: Vec<ArgMetadata>,
    handler: handler::CommandHandler,
  ) -> Self {
    log::trace!(
      "creating command metadata: desc='{}', perm={:?}, reply={:?}, args={:?}",
      desc,
      perm,
      reply,
      args
    );
    Self {
      perm,
      desc,
      reply,
      args,
      handler,
    }
  }
}
