use rusqlite::params;
use std::sync::Arc;
use tokio::sync::Mutex;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use teloxide::prelude::UserId;

use super::types::{Permission, PermissionMap};

/// A thread-safe permission management system backed by SQLite.
///
/// This structure provides a complete interface for creating, modifying,
/// and retrieving permission flags associated with Telegram users.
/// Permissions are stored persistently and safely accessible across threads.
#[derive(Debug, Clone)]
pub struct PermissionManager {
  pub db: Arc<Pool<SqliteConnectionManager>>,
}

impl PermissionManager {
  /// Creates a new [`PermissionManager`] instance using the given SQLite connection pool.
  ///
  /// This function automatically initializes the schema if it does not exist yet,
  /// creating the `permissions` table in the database.
  ///
  /// # Returns
  /// A new `PermissionManager` wrapped in [`anyhow::Result`].
  ///
  /// # Errors
  /// Returns an error if the database connection or schema initialization fails.
  pub fn new(db: Arc<Pool<SqliteConnectionManager>>) -> anyhow::Result<Self> {
    let mgr = Self { db };
    mgr.init_schema()?;
    Ok(mgr)
  }

  /// Creates a shared, asynchronously safe [`PermissionManager`] instance.
  ///
  /// This version wraps the `PermissionManager` inside an [`Arc`] and [`Mutex`],
  /// making it suitable for use in async contexts such as Telegram bot handlers.
  ///
  /// # Returns
  /// Returns an `Arc<Mutex<Self>>` for concurrent use across async tasks.
  ///
  /// # Errors
  /// Returns an error if initialization of the internal schema fails.
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

  /// Removes all stored permissions for a given user.
  ///
  /// If the user does not exist in the table, this operation silently does nothing.
  /// It’s commonly used when revoking all access rights for a user completely.
  ///
  /// # Parameters
  /// * `user_id` – Telegram user identifier.
  ///
  /// # Returns
  /// Returns `Ok(())` on success.
  ///
  /// # Errors
  /// Returns an error if the database query fails.
  pub fn reset(
    &self,
    user_id: UserId,
  ) -> anyhow::Result<()> {
    let conn = self.db.get()?;
    conn.execute(
      "DELETE FROM permissions WHERE user_id = ?1",
      params![user_id.0],
    )?;

    log::trace!("removed all permissions for user {}", user_id);
    Ok(())
  }

  /// Clears all permission entries from the database.
  ///
  /// This operation deletes all rows from the `permissions` table,
  /// effectively resetting the permission state for all users.
  ///
  /// # Returns
  /// Returns `Ok(())` when the operation completes successfully.
  ///
  /// # Errors
  /// Returns an error if the database connection or execution fails.
  pub fn clear(&self) -> anyhow::Result<()> {
    let conn = self.db.get()?;
    conn.execute("DELETE FROM permissions", [])?;
    Ok(())
  }

  /// Retrieves the permission flags for a specific user.
  ///
  /// If the user is not found, the function returns an error.
  /// The returned [`Permission`] represents the user’s full permission set.
  ///
  /// # Returns
  /// A valid [`Permission`] object for the specified user.
  ///
  /// # Errors
  /// Returns an error if the user is missing or if database access fails.
  pub fn get(
    &self,
    user_id: UserId,
  ) -> anyhow::Result<Permission> {
    let conn = self.db.get()?;
    let perm = conn.query_row(
      "SELECT flags FROM permissions WHERE user_id = ?1",
      params![user_id.0],
      |row| Ok(Permission::from_bits_truncate(row.get::<_, u32>(0)?)),
    )?;

    log::trace!("get permission for user {}: {:?}", user_id, perm);

    Ok(perm)
  }

  /// Sets the exact permission value for a specific user.
  ///
  /// If the user already exists, the value is updated; otherwise, it is inserted.
  /// This function should be used for explicit permission assignments.
  ///
  /// # Parameters
  /// * `user_id` – Telegram user identifier.
  /// * `perm` – The permission flags to assign.
  ///
  /// # Returns
  /// Returns `Ok(())` on success.
  ///
  /// # Errors
  /// Returns an error if the database query fails.
  pub fn set(
    &self,
    user_id: UserId,
    perm: Permission,
  ) -> anyhow::Result<()> {
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

  /// Grants one or more permissions to a user without removing existing ones.
  ///
  /// This operation merges the current permission set with the new flags.
  /// Useful when incrementally adding privileges to users.
  ///
  /// # Returns
  /// Returns `Ok(())` on success.
  ///
  /// # Errors
  /// Returns an error if retrieving or updating permissions fails.
  pub fn grant(
    &self,
    user_id: UserId,
    perm: Permission,
  ) -> anyhow::Result<()> {
    let current = self.get(user_id).unwrap_or(Permission::NONE);
    self.set(user_id, current | perm)?;

    log::trace!(
      "grant permission {:?} to user {}, previous {:?}",
      perm,
      user_id,
      current
    );

    Ok(())
  }

  /// Revokes specific permissions from a user.
  ///
  /// This removes only the specified flags, leaving other permissions intact.
  /// If the user doesn’t have those permissions, nothing changes.
  ///
  /// # Returns
  /// Returns `Ok(())` when successful.
  ///
  /// # Errors
  /// Returns an error if any database operation fails.
  pub fn revoke(
    &self,
    user_id: UserId,
    perm: Permission,
  ) -> anyhow::Result<()> {
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

  /// Checks whether a user explicitly has a given permission.
  ///
  /// This function compares the user's current flags with the requested one.
  /// Returns `true` if all required flags are present.
  ///
  /// # Returns
  /// A boolean wrapped in [`anyhow::Result`] indicating whether the user has the permission.
  ///
  /// # Errors
  /// Returns an error if reading user permissions fails.
  pub fn has(
    &self,
    user_id: UserId,
    perm: Permission,
  ) -> anyhow::Result<bool> {
    let has_perm = self.get(user_id)?.contains(perm);

    log::trace!(
      "check if user {} has permission {:?}: {}",
      user_id,
      perm,
      has_perm
    );

    Ok(has_perm)
  }

  /// Checks whether a user’s permission level meets or exceeds a threshold.
  ///
  /// This method compares permission "levels" rather than exact bit matches.
  /// It is suitable for hierarchical permission systems (e.g., admin ≥ user).
  ///
  /// # Returns
  /// Returns `true` if the user’s permission level allows access.
  ///
  /// # Errors
  /// Returns an error if reading user permissions fails.
  pub fn can(
    &self,
    user_id: UserId,
    perm: Permission,
  ) -> anyhow::Result<bool> {
    let can_access = self.get(user_id)?.level() >= perm.level();

    log::trace!(
      "check if user {} can access level {:?}: {}",
      user_id,
      perm,
      can_access
    );

    Ok(can_access)
  }

  /// Iterates over all stored permissions and returns them as a vector.
  ///
  /// This method reads every `(user_id, Permission)` pair from the database.
  /// Invalid rows are logged and skipped.
  ///
  /// # Returns
  /// A vector of `(UserId, Permission)` tuples.
  ///
  /// # Errors
  /// Returns an error if database reading fails.
  pub fn perm_iter(&self) -> anyhow::Result<Vec<(UserId, Permission)>> {
    let conn = self.db.get()?;
    let mut stmt = conn.prepare("SELECT user_id, flags FROM permissions")?;

    let rows = stmt.query_map([], |row| {
      let _user_id: String = row.get(0)?;
      let user_id: u64 = _user_id.parse().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
          0,
          rusqlite::types::Type::Text,
          Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("parsing error: {}", e),
          )),
        )
      })?;
      let flags: u32 = row.get(1)?;
      Ok((UserId(user_id), Permission::from_bits_truncate(flags)))
    })?;

    let result: Vec<_> = rows
      .filter_map(|r| match &r {
        Ok(_val) => Some(r.unwrap()),
        Err(e) => {
          log::error!("error while reading row: {:?}", e);
          None
        }
      })
      .collect();

    Ok(result)
  }

  /// Loads a snapshot of permissions into the database.
  ///
  /// The snapshot represents a serialized state of permissions that will
  /// be inserted into the table. Existing data remains unless cleared manually.
  ///
  /// # Parameters
  /// * `snapshot` – A map of users and their permissions.
  ///
  /// # Returns
  /// Returns `Ok(())` if all rows were inserted successfully.
  ///
  /// # Errors
  /// Returns an error if any insert operation fails.
  pub fn load_snapshot_iter(
    &self,
    snapshot: &PermissionMap,
  ) -> anyhow::Result<()> {
    let conn = self.db.get()?;

    for (user_id, perm) in snapshot {
      conn.execute(
        "INSERT INTO permissions (user_id, flags) VALUES (?1, ?2)",
        params![user_id.0, perm.bits()],
      )?;
    }

    Ok(())
  }

  /// Creates an in-memory snapshot of all current permissions.
  ///
  /// Useful for backups or exporting permission states between bot sessions.
  /// The resulting [`PermissionMap`] can later be reloaded using [`load_snapshot`].
  ///
  /// # Returns
  /// A [`PermissionMap`] containing all `(UserId, Permission)` pairs.
  ///
  /// # Errors
  /// Returns an error if fetching data from the database fails.
  pub fn snapshot(&self) -> anyhow::Result<PermissionMap> {
    let result: PermissionMap = self.perm_iter()?.into_iter().collect();
    log::trace!("snapshot() returned {} entries", result.len());
    Ok(result)
  }

  /// Replaces all existing permissions with a given snapshot.
  ///
  /// This function clears the entire permission table and then loads
  /// the provided snapshot, effectively restoring a previous state.
  ///
  /// # Parameters
  /// * `snapshot` – The permission map to load into the database.
  ///
  /// # Returns
  /// Returns `Ok(())` on successful replacement.
  ///
  /// # Errors
  /// Returns an error if any clear or insert operation fails.
  pub fn load_snapshot(
    &self,
    snapshot: &PermissionMap,
  ) -> anyhow::Result<()> {
    self.clear()?;
    self.load_snapshot_iter(snapshot)?;
    Ok(())
  }
}
