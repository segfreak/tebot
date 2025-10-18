use serde::{Deserialize, Serialize};

use bitflags::bitflags;
use std::collections::HashMap;
use teloxide::prelude::UserId;

pub type PermissionMap = HashMap<UserId, Permission>;

bitflags! {
  #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
  pub struct Permission: u32 {
    const NONE  = 0b0000;
    const USER  = 0b0001;
    const ADMIN = 0b0010;
    const OWNER = 0b0100;
  }
}

impl Permission {
  pub fn level(&self) -> u8 {
    if self.contains(Permission::OWNER) {
      3
    } else if self.contains(Permission::ADMIN) {
      2
    } else if self.contains(Permission::USER) {
      1
    } else {
      0
    }
  }
}
