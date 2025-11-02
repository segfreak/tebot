use serde::{Deserialize, Serialize};

use bitflags::bitflags;
use std::collections::HashMap;
use teloxide::prelude::UserId;

/// A map associating Telegram users with their corresponding permissions.
///
/// This type alias simplifies working with collections of permissions.
/// It is primarily used when taking snapshots of permission states or
/// bulk-loading user permissions from persistent storage.
///
/// Each entry maps a [`UserId`] to a [`Permission`] bitmask.
pub type PermissionMap = HashMap<UserId, Permission>;

bitflags! {
  /// Bitflag-based representation of user permissions.
  ///
  /// Each variant defines a specific access level or privilege assigned to users.
  /// The permissions are composable and can be combined using bitwise OR (`|`)
  /// and removed using bitwise subtraction (`-`), which is internally translated
  /// into XOR masking for convenience.
  ///
  /// # Variants
  /// - `NONE`: No permissions granted.
  /// - `USER`: Basic user-level access.
  /// - `ADMIN`: Elevated administrative privileges.
  /// - `OWNER`: Full control, typically reserved for the bot owner or system root.
  ///
  /// # Serialization
  /// This structure supports [`serde`] serialization and deserialization for
  /// seamless persistence in JSON or database records.
  #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
  pub struct Permission: u32 {
    /// No permissions assigned.
    ///
    /// Used as a default value when a user has no registered privileges.
    const NONE  = 0b0000;

    /// Standard user-level permission.
    ///
    /// This flag allows access to basic bot commands and general functionality.
    const USER  = 0b0001;

    /// Administrator-level permission.
    ///
    /// Grants access to management functions, such as moderation or system actions.
    const ADMIN = 0b0010;

    /// Owner-level permission.
    ///
    /// Represents the highest level of privilege, often reserved for the bot creator.
    /// Owners can override all restrictions, regardless of other permissions.
    const OWNER = 0b0100;
  }
}

impl Permission {
  /// Returns the hierarchical level of the current permission state.
  ///
  /// This function converts the permission bitmask into a comparable numeric value,
  /// enabling simple comparisons such as `>=` or `<` between permission levels.
  ///
  /// The level mapping is defined as:
  /// - `OWNER` → 3
  /// - `ADMIN` → 2
  /// - `USER` → 1
  /// - `NONE` → 0
  ///
  /// # Examples
  /// ```
  /// use crate::types::Permission;
  ///
  /// let p = Permission::ADMIN;
  /// assert_eq!(p.level(), 2);
  /// ```
  ///
  /// # Returns
  /// A numeric permission level corresponding to the highest flag set.
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
