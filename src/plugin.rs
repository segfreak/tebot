use indexmap::IndexMap;

use super::command;
use super::handler;

pub type PluginBox = Box<dyn Plugin>;
pub type PluginMap = IndexMap<String, PluginBox>;

pub trait Plugin: Send + Sync {
  fn name(&self) -> &str;
  fn commands(&self) -> IndexMap<String, command::CommandMetadata>;
  fn update_handlers(&self) -> Vec<handler::UpdateHandler>;
}
