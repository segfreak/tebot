use teloxide::prelude::UserId;

use super::permissions::{Permission, PermissionMap};

pub fn parse_permission(s: &str) -> Result<Permission, String> {
  let s = s.trim();
  if s.is_empty() {
    return Err("Empty role string".to_string());
  }
  let mut role_mask = Permission::NONE;
  for role_str in s.split('|') {
    match role_str.to_uppercase().trim() {
      "USER" => role_mask |= Permission::USER,
      "ADMIN" => role_mask |= Permission::ADMIN,
      "OWNER" => role_mask |= Permission::OWNER,
      other => return Err(format!("Unknown role: {}", other)),
    }
  }
  Ok(role_mask)
}

pub fn parse_uid(s: &str) -> Result<UserId, String> {
  s.parse::<u64>()
    .map(UserId)
    .map_err(|_| format!("Invalid user ID: '{}'", s))
}

pub fn parse_uid_perm(s: &str) -> Result<(UserId, Permission), String> {
  let parts: Vec<&str> = s.trim().split_whitespace().collect();
  if parts.len() != 2 {
    log::error!("Invalid role entry: {}", s);
    return Err(format!("Invalid role entry: {}", s));
  }

  let user_id = parse_uid(parts[0])?;
  let role_mask = parse_permission(parts[1])?;

  Ok((user_id, role_mask))
}

pub fn parse_perm_arg(s: &str) -> Result<PermissionMap, String> {
  let mut map = PermissionMap::new();
  let (uid, role) = parse_uid_perm(s)?;
  map.insert(uid, role);
  Ok(map)
}
