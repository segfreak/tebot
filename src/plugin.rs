use std::sync::Arc;

use indexmap::IndexMap;
use tokio::sync::Mutex;

use crate::dispatcher::Dispatcher;

use super::command;
use super::handler;

pub type PluginBox = Box<dyn Plugin>;
pub type PluginMap = IndexMap<String, PluginBox>;

pub trait Plugin: Send + Sync {
  fn name(&self) -> &str;
  fn commands(&self) -> IndexMap<String, command::CommandMetadata>;
  fn update_handlers(&self) -> Vec<handler::UpdateHandler>;
}

pub async fn register_all(dp: Arc<Mutex<Dispatcher>>, plugs: Vec<PluginBox>) {
  for plug in plugs {
    dp.lock().await.register_plugin(plug).await;
  }
}
