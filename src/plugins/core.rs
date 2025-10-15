use indexmap::IndexMap;

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
    IndexMap::new()
  }
}

pub fn get_plugin() -> Box<dyn plugin::Plugin> {
  Box::new(CorePlugin::new())
}
