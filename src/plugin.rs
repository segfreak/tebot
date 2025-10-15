use indexmap::IndexMap;

use super::command;

pub trait Plugin {
  fn name(&self) -> &str;
  fn commands(&self) -> IndexMap<String, command::CommandMetadata>;
}
