pub mod access;
pub mod core;

use crate::plugin;

pub async fn all() -> Vec<plugin::PluginBox> {
  vec![core::get_plugin().await, access::get_plugin().await]
}
