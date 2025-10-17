pub mod access;
pub mod core;

use crate::plugin;

pub fn all() -> Vec<plugin::PluginBox> {
  vec![core::get_plugin(), access::get_plugin()]
}
