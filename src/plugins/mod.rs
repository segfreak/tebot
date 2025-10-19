pub mod access;
pub mod core;
pub mod sigthief;
pub mod storage;
pub mod system;
pub mod time;

use crate::bot::plugin;

pub async fn all() -> Vec<plugin::PluginBox> {
  vec![
    core::get_plugin(),
    access::get_plugin(),
    time::get_plugin(),
    system::get_plugin(),
    sigthief::get_plugin(),
    storage::get_plugin(),
  ]
}
