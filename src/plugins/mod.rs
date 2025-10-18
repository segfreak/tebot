pub mod access;
pub mod core;
pub mod system;
pub mod time;

use crate::bot::plugin;

pub async fn all() -> Vec<plugin::PluginBox> {
  vec![
    core::get_plugin().await,
    access::get_plugin().await,
    time::get_plugin().await,
    system::get_plugin().await,
  ]
}
