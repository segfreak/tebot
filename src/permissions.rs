use serde::{Deserialize, Serialize};

use rusqlite::params;
use std::sync::{Arc, Mutex};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use bitflags::bitflags;

use teloxide::prelude::UserId;

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
  pub fn new(db: Arc<Pool<SqliteConnectionManager>>) -> Self {
    let mgr = Self { db };
    mgr.init_schema();
    mgr
  }

  pub fn new_arc_mutex(db: Arc<Pool<SqliteConnectionManager>>) -> Arc<Mutex<Self>> {
    Arc::new(Mutex::new(Self::new(db)))
  }

  fn init_schema(&self) {
    let conn = self.db.get().unwrap();
    conn
      .execute(
        "CREATE TABLE IF NOT EXISTS permissions (
                user_id TEXT PRIMARY KEY,
                flags   INTEGER NOT NULL
            )",
        [],
      )
      .unwrap();
  }

  pub fn get(&self, user_id: UserId) -> Permission {
    let conn = self.db.get().unwrap();
    conn
      .query_row(
        "SELECT flags FROM permissions WHERE user_id = ?1",
        params![user_id.0],
        |row| Ok(Permission::from_bits_truncate(row.get::<_, u32>(0)?)),
      )
      .unwrap_or(Permission::NONE)
  }

  pub fn set(&self, user_id: UserId, perm: Permission) {
    let conn = self.db.get().unwrap();
    conn
      .execute(
        "INSERT INTO permissions (user_id, flags)
             VALUES (?1, ?2)
             ON CONFLICT(user_id) DO UPDATE SET flags = excluded.flags",
        params![user_id.0, perm.bits()],
      )
      .unwrap();
  }

  pub fn grant(&self, user_id: UserId, perm: Permission) {
    let current = self.get(user_id);
    self.set(user_id, current | perm);
  }

  pub fn revoke(&self, user_id: UserId, perm: Permission) {
    let current = self.get(user_id);
    self.set(user_id, current - perm);
  }

  pub fn has(&self, user_id: UserId, perm: Permission) -> bool {
    self.get(user_id).contains(perm)
  }
}
