use derivative::Derivative;

use super::handler;
use crate::permissions::types::Permission;

/// Represents a parsed bot command extracted from a message string.
///
/// A command consists of:
/// - A `prefix` (e.g. `'/'`, `'!'`)
/// - A `name` (e.g. `"start"`, `"help"`)
/// - A list of `args` (arguments separated by spaces or enclosed in quotes)
///
/// This structure is typically produced by [`Command::with_prefix`] or
/// [`Command::with_prefixes`] and used during command dispatch or validation.
pub struct Command {
  /// The character that identifies the beginning of a command.
  pub prefix: char,

  /// The name of the command (e.g., `"help"` or `"ban"`).
  pub name: String,

  /// List of parsed arguments following the command name.
  pub args: Vec<String>,
}

impl Command {
  /// Attempts to parse a command string using the specified prefix.
  ///
  /// This method recognizes quoted arguments (e.g. `"hello world"`) as single tokens
  /// and ignores leading whitespace. It returns `None` if the string does not
  /// start with the given prefix or if no valid command name is found.
  ///
  /// # Examples
  /// ```
  /// use crate::command::Command;
  /// let cmd = Command::with_prefix("/say hello", '/').unwrap();
  /// assert_eq!(cmd.name, "say");
  /// assert_eq!(cmd.args, vec!["hello"]);
  /// ```
  pub fn with_prefix(
    s: &str,
    prefix: char,
  ) -> Option<Self> {
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
        log::trace!("no command name found in string '{}'", s);
        return None;
      }
    };
    let args = parts.into_iter().skip(1).collect();

    log::trace!("parsed command '{}' with args {:?}", name, args);
    Some(Self { prefix, name, args })
  }

  /// Parses a command string using any of the provided allowed prefixes.
  ///
  /// This method checks whether the first character of the input string matches
  /// one of the allowed prefix characters (e.g., `'/'`, `'!'`, or `'.'`).
  /// If a match is found, the command is parsed with that prefix.
  ///
  /// # Examples
  /// ```
  /// use crate::command::Command;
  /// let allowed = vec!['/', '!'];
  /// assert!(Command::with_prefixes("!ping", allowed).is_some());
  /// assert!(Command::with_prefixes("ping", allowed).is_none());
  /// ```
  pub fn with_prefixes<T>(
    s: &str,
    allowed: T,
  ) -> Option<Self>
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

/// Defines whether a command argument is required or context-dependent.
///
/// Argument requirements determine when a command may be executed and
/// what message context (e.g., reply or non-reply) is needed for proper parsing.
#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ArgRequirement {
  /// The argument is optional and may be omitted.
  Optional,

  /// The argument must only appear if the command is used as a reply to another message.
  OnlyWithReply,

  /// The argument must only appear if the command is *not* a reply.
  OnlyWithoutReply,

  /// The argument is mandatory and must always be provided.
  Required,
}

/// Defines reply-specific requirements for command execution.
///
/// Some commands may require a reply context (e.g. replying to a user's message)
/// while others may forbid it or treat it as optional.
#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ReplyRequirement {
  /// No reply context is required.
  None,

  /// Command can only be used when replying to a message without a document.
  OnlyWithoutDocument,

  /// Reply is optional; the command can be executed with or without one.
  Optional,

  /// A reply context is strictly required for this command.
  Required,
}

/// Metadata describing a single command argument.
///
/// Stores its name, description, and requirement level.  
/// Primarily used to generate help messages and perform argument validation.
#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct ArgMetadata {
  /// Argument name (e.g., `"username"` or `"amount"`).
  pub name: String,

  /// Human-readable description explaining the argument’s purpose.
  pub description: String,

  /// Defines whether the argument is optional, required, or context-sensitive.
  pub requirement: ArgRequirement,
}

impl ArgMetadata {
  /// Creates a new [`ArgMetadata`] instance with the specified properties.
  ///
  /// This is a simple helper used to define command argument structures
  /// when registering commands in a command registry or handler map.
  ///
  /// # Examples
  /// ```
  /// use crate::command::{ArgMetadata, ArgRequirement};
  ///
  /// let arg = ArgMetadata::new(
  ///     "target".to_string(),
  ///     "Specifies the user to ban".to_string(),
  ///     ArgRequirement::Required
  /// );
  /// assert_eq!(arg.name, "target");
  /// ```
  pub fn new(
    name: String,
    description: String,
    requirement: ArgRequirement,
  ) -> Self {
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

/// Metadata describing a registered bot command.
///
/// This structure combines command documentation, argument metadata,
/// permission requirements, and a runtime handler function used to execute
/// the command logic once invoked by the bot.
///
/// Instances of this type are usually stored in a command registry or routing table.
#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub struct CommandMetadata {
  /// The minimum required permission to execute this command.
  pub perm: Permission,

  /// Short textual description of what the command does.
  pub desc: String,

  /// Defines whether this command requires or forbids a reply context.
  pub reply: ReplyRequirement,

  /// List of argument definitions describing expected command parameters.
  pub args: Vec<ArgMetadata>,

  /// The runtime handler responsible for executing the command.
  #[derivative(Debug = "ignore")]
  pub handler: handler::CommandHandler,
}

impl CommandMetadata {
  /// Creates a new [`CommandMetadata`] object.
  ///
  /// This function ties together all command properties into a single unit
  /// that can be registered and executed by the command dispatcher.
  ///
  /// # Parameters
  /// - `perm`: The permission level required to access the command.
  /// - `desc`: A description displayed in help menus or documentation.
  /// - `reply`: Specifies whether replies are mandatory or optional.
  /// - `args`: A list of expected arguments.
  /// - `handler`: The actual handler function to execute when the command runs.
  pub fn new(
    perm: Permission,
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
