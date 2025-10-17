use serde::{Deserialize, Serialize};

use rusqlite::params;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use bitflags::bitflags;

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

#[derive(Debug, Clone)]
pub struct PermissionManager {
  pub db: Arc<Pool<SqliteConnectionManager>>,
}

impl PermissionManager {
  pub fn new(db: Arc<Pool<SqliteConnectionManager>>) -> anyhow::Result<Self> {
    let mgr = Self { db };
    mgr.init_schema()?;
    Ok(mgr)
  }

  pub fn new_shared(db: Arc<Pool<SqliteConnectionManager>>) -> anyhow::Result<Arc<Mutex<Self>>> {
    let mgr = Self::new(db)?;
    Ok(Arc::new(Mutex::new(mgr)))
  }

  fn init_schema(&self) -> anyhow::Result<()> {
    let conn = self.db.get()?;
    conn.execute(
      "CREATE TABLE IF NOT EXISTS permissions (
                user_id TEXT PRIMARY KEY,
                flags   INTEGER NOT NULL
            )",
      [],
    )?;
    Ok(())
  }

  pub fn reset(&self, user_id: UserId) -> anyhow::Result<()> {
    let conn = self.db.get()?;
    conn.execute(
      "DELETE FROM permissions WHERE user_id = ?1",
      params![user_id.0],
    )?;

    log::trace!("removed all permissions for user {}", user_id);
    Ok(())
  }

  pub fn clear(&self) -> anyhow::Result<()> {
    let conn = self.db.get()?;
    conn.execute("DELETE FROM permissions", [])?;
    Ok(())
  }

  pub fn get(&self, user_id: UserId) -> anyhow::Result<Permission> {
    let conn = self.db.get()?;
    let perm = conn.query_row(
      "SELECT flags FROM permissions WHERE user_id = ?1",
      params![user_id.0],
      |row| Ok(Permission::from_bits_truncate(row.get::<_, u32>(0)?)),
    )?;

    log::trace!("get permission for user {}: {:?}", user_id, perm);

    Ok(perm)
  }

  pub fn set(&self, user_id: UserId, perm: Permission) -> anyhow::Result<()> {
    let conn = self.db.get()?;
    conn.execute(
      "INSERT INTO permissions (user_id, flags)
             VALUES (?1, ?2)
             ON CONFLICT(user_id) DO UPDATE SET flags = excluded.flags",
      params![user_id.0, perm.bits()],
    )?;

    log::trace!("set permission for user {}: {:?}", user_id, perm);

    Ok(())
  }

  pub fn grant(&self, user_id: UserId, perm: Permission) -> anyhow::Result<()> {
    let current = self.get(user_id)?;
    self.set(user_id, current | perm)?;

    log::trace!(
      "grant permission {:?} to user {}, previous {:?}",
      perm,
      user_id,
      current
    );

    Ok(())
  }

  pub fn revoke(&self, user_id: UserId, perm: Permission) -> anyhow::Result<()> {
    let current = self.get(user_id)?;
    self.set(user_id, current - perm)?;

    log::trace!(
      "revoke permission {:?} from user {}, previous {:?}",
      perm,
      user_id,
      current
    );

    Ok(())
  }

  pub fn has(&self, user_id: UserId, perm: Permission) -> anyhow::Result<bool> {
    let has_perm = self.get(user_id)?.contains(perm);

    log::trace!(
      "check if user {} has permission {:?}: {}",
      user_id,
      perm,
      has_perm
    );

    Ok(has_perm)
  }

  pub fn can(&self, user_id: UserId, perm: Permission) -> anyhow::Result<bool> {
    let can_access = self.get(user_id)?.level() >= perm.level();

    log::trace!(
      "check if user {} can access level {:?}: {}",
      user_id,
      perm,
      can_access
    );

    Ok(can_access)
  }

  pub fn perm_iter(&self) -> anyhow::Result<Vec<(UserId, Permission)>> {
    let conn = self.db.get()?;
    let mut stmt = conn.prepare("SELECT user_id, flags FROM permissions")?;

    let rows = stmt.query_map([], |row| {
      let user_id: u64 = row.get(0)?;
      let flags: u32 = row.get(1)?;
      Ok((UserId(user_id), Permission::from_bits_truncate(flags)))
    })?;

    Ok(rows.filter_map(Result::ok).collect())
  }

  pub fn load_snapshot_iter(&self, snapshot: &PermissionMap) -> anyhow::Result<()> {
    let conn = self.db.get()?;

    for (user_id, perm) in snapshot {
      conn.execute(
        "INSERT INTO permissions (user_id, flags) VALUES (?1, ?2)",
        params![user_id.0, perm.bits()],
      )?;
    }

    Ok(())
  }

  pub fn snapshot(&self) -> anyhow::Result<PermissionMap> {
    Ok(self.perm_iter()?.into_iter().collect())
  }

  pub fn load_snapshot(&self, snapshot: &PermissionMap) -> anyhow::Result<()> {
    self.clear()?;
    self.load_snapshot_iter(snapshot)?;
    Ok(())
  }
}
