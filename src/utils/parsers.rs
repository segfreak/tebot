use anyhow::{anyhow, Context};
use teloxide::prelude::UserId;

use crate::permissions::types::{Permission, PermissionMap};

/// Parses a permission string into a bitmask of [`Permission`].
///
/// Accepts a string where multiple roles are separated by `|` (e.g. `"USER|ADMIN"`).
/// Each token is converted to uppercase and trimmed before processing.
/// Returns an error if any unknown role is encountered.
/// Useful for parsing configuration files, CLI arguments, or environment variables.
///
/// Example: `"ADMIN|OWNER"` → `Permission::ADMIN | Permission::OWNER`.
pub async fn parse_permission(s: &str) -> anyhow::Result<Permission> {
  let s = s.trim();
  log::trace!("parsing permission from string '{}'", s);

  if s.is_empty() {
    log::trace!("permission string is empty");
    return Err(anyhow!("empty permission string"));
  }

  let mut role_mask = Permission::NONE;
  for role_str in s.split('|') {
    match role_str.to_uppercase().trim() {
      "USER" => {
        role_mask |= Permission::USER;
        log::trace!("added 'user' permission");
      }
      "ADMIN" => {
        role_mask |= Permission::ADMIN;
        log::trace!("added 'admin' permission");
      }
      "OWNER" => {
        role_mask |= Permission::OWNER;
        log::trace!("added 'owner' permission");
      }
      other => {
        log::trace!("unknown permission '{}'", other);
        return Err(anyhow!("unknown permission: {}", other));
      }
    }
  }

  log::trace!("resulting permission mask: {:?}", role_mask);
  Ok(role_mask)
}

/// Parses a user ID string into a [`UserId`] instance.
///
/// Expects a numeric string (e.g. `"123456789"`) and converts it into `u64`,
/// then wraps it in [`UserId`].  
/// Returns an error if the string contains invalid characters
/// or cannot be parsed as an unsigned integer.
/// Commonly used for loading Telegram user IDs from configuration or databases.
pub async fn parse_uid(s: &str) -> anyhow::Result<UserId> {
  log::trace!("parsing user id from string '{}'", s);
  s.parse::<u64>()
    .map(UserId)
    .map_err(|_| anyhow!("invalid user id: '{}'", s))
}

/// Parses a string in the format `"uid permission"` into a `(UserId, Permission)` pair.
///
/// Expects two space-separated values: a user ID and a permission string
/// (e.g. `"123 ADMIN|OWNER"`).  
/// Validates both parts and returns a tuple on success.
/// Produces a detailed error if the format is invalid or parsing fails.
/// Useful for parsing structured configuration entries or command-line input.
pub async fn parse_uid_perm(s: &str) -> anyhow::Result<(UserId, Permission)> {
  log::trace!("parsing user id and permission from '{}'", s);
  let parts: Vec<&str> = s.trim().split_whitespace().collect();
  if parts.len() != 2 {
    log::trace!("invalid role entry '{}'", s);
    return Err(anyhow!("invalid role entry: {}", s));
  }

  let user_id = parse_uid(parts[0])
    .await
    .context("parsing user id failed")?;
  let role_mask = parse_permission(parts[1])
    .await
    .context("parsing permission failed")?;

  log::trace!(
    "parsed entry: user_id={:?}, permission={:?}",
    user_id,
    role_mask
  );
  Ok((user_id, role_mask))
}

/// Converts a `"uid permission"` string into a [`PermissionMap`].
///
/// Internally calls [`parse_uid_perm`] to extract the `(UserId, Permission)` pair,
/// then inserts it into a new map.  
/// Useful for initializing permission mappings from command-line arguments
/// or config entries containing a single user-role definition.
/// Emits detailed trace logs for debugging and returns the constructed map.
pub async fn parse_perm_arg(s: &str) -> anyhow::Result<PermissionMap> {
  log::trace!("parsing permission map from '{}'", s);
  let mut map = PermissionMap::new();
  let (uid, role) = parse_uid_perm(s).await?;
  map.insert(uid, role);
  log::trace!("permission map created: {:?}", map);
  Ok(map)
}
