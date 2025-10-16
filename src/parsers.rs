use teloxide::prelude::UserId;

use super::permissions::{Permission, PermissionMap};

pub fn parse_permission(s: &str) -> Result<Permission, String> {
  let s = s.trim();
  log::trace!("parsing permission from string '{}'", s);

  if s.is_empty() {
    log::trace!("permission string is empty");
    return Err("empty role string".to_string());
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
        return Err(format!("unknown role: {}", other));
      }
    }
  }

  log::trace!("resulting permission mask: {:?}", role_mask);
  Ok(role_mask)
}

pub fn parse_uid(s: &str) -> Result<UserId, String> {
  log::trace!("parsing user id from string '{}'", s);
  s.parse::<u64>().map(UserId).map_err(|_| {
    log::trace!("failed to parse user id from '{}'", s);
    format!("invalid user id: '{}'", s)
  })
}

pub fn parse_uid_perm(s: &str) -> Result<(UserId, Permission), String> {
  log::trace!("parsing user id and permission from '{}'", s);
  let parts: Vec<&str> = s.trim().split_whitespace().collect();
  if parts.len() != 2 {
    log::trace!("invalid role entry '{}'", s);
    return Err(format!("invalid role entry: {}", s));
  }

  let user_id = parse_uid(parts[0])?;
  let role_mask = parse_permission(parts[1])?;

  log::trace!(
    "parsed entry: user_id={:?}, permission={:?}",
    user_id,
    role_mask
  );
  Ok((user_id, role_mask))
}

pub fn parse_perm_arg(s: &str) -> Result<PermissionMap, String> {
  log::trace!("parsing permission map from '{}'", s);
  let mut map = PermissionMap::new();
  let (uid, role) = parse_uid_perm(s)?;
  map.insert(uid, role);
  log::trace!("permission map created: {:?}", map);
  Ok(map)
}
