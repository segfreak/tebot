use std::sync::Arc;

use indexmap::IndexMap;
use tokio::sync::Mutex;

use super::dispatcher::Dispatcher;

use super::command;
use super::handler;

/// A boxed plugin trait object.
pub type PluginBox = Box<dyn Plugin>;

/// Mapping of plugin names to their plugin instances.
pub type PluginMap = IndexMap<String, PluginBox>;

pub trait Plugin: Send + Sync {
  /// Returns the plugin's name.
  fn name(&self) -> &str;

  /// Returns all commands exposed by this plugin.
  fn commands(&self) -> IndexMap<String, command::CommandMetadata>;

  /// Returns update handlers registered by this plugin.
  fn update_handlers(&self) -> Vec<handler::UpdateHandler>;
}

/// Registers multiple plugins into a dispatcher.
///
/// Each plugin is added to the dispatcher, including its commands
/// and update handlers.
pub async fn register_all(
  dp: Arc<Mutex<Dispatcher>>,
  plugs: Vec<PluginBox>,
) {
  for plug in plugs {
    let name = plug.name().to_string();
    log::debug!("registering plugin {}", name);
    dp.lock().await.register_plugin(plug).await;
    log::debug!("plugin {} successfully registered", name);
  }
}
