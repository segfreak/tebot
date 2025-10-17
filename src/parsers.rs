use anyhow::{anyhow, Context};
use teloxide::prelude::UserId;

use super::permissions::{Permission, PermissionMap};

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

pub async fn parse_uid(s: &str) -> anyhow::Result<UserId> {
  log::trace!("parsing user id from string '{}'", s);
  s.parse::<u64>()
    .map(UserId)
    .map_err(|_| anyhow!("invalid user id: '{}'", s))
}

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

pub async fn parse_perm_arg(s: &str) -> anyhow::Result<PermissionMap> {
  log::trace!("parsing permission map from '{}'", s);
  let mut map = PermissionMap::new();
  let (uid, role) = parse_uid_perm(s).await?;
  map.insert(uid, role);
  log::trace!("permission map created: {:?}", map);
  Ok(map)
}
