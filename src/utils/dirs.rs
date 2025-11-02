use std::path::PathBuf;

use super::env;
use crate::bot::plugin;

/// Ensures that the given directory exists, creating it if necessary.
///
/// This function is asynchronous and will recursively create
/// any missing parent directories as well.  
///
/// Returns the same path once the directory has been verified
/// or successfully created.
///
/// # Errors
/// Returns an error if the directory cannot be created due to
/// filesystem permissions or other I/O issues.
pub async fn ensure_exists(dir: PathBuf) -> anyhow::Result<PathBuf> {
  tokio::fs::create_dir_all(&dir).await?;
  Ok(dir)
}

/// Returns the root directory used for storing persistent bot data.
///
/// The location is determined by environment configuration and may vary
/// depending on the system or user settings.  
///
/// This directory acts as the base for all bot-related files, including
/// plugin data, caches, and temporary storage.
///
/// The directory itself is not created automatically.
pub async fn root_data_dir() -> PathBuf {
  PathBuf::from(env::get_data_dir().await)
}

/// Returns the path to a specific subdirectory inside the bot’s data root.
///
/// This function only constructs the path and does not create the directory.
/// It is typically used to prepare a structured file layout for bot modules.  
///
/// Use [`ensure_exists`] if you need to make sure the path actually exists.
///
/// The resulting path is always relative to the main bot data directory.
pub async fn sub_data_dir(sub: &str) -> PathBuf {
  root_data_dir().await.join(sub)
}

/// Returns (and ensures) the data directory specific to the given plugin.
///
/// Each plugin has its own dedicated storage space for configuration,
/// logs, or persistent state files.  
///
/// This function will automatically create the directory if it does
/// not already exist.
///
/// # Errors
/// Returns an error if the directory cannot be created or accessed.
pub async fn plugin_data_dir(plugin: plugin::PluginBox) -> anyhow::Result<PathBuf> {
  ensure_exists(sub_data_dir(plugin.name()).await).await
}

/// Returns either the root data directory or a subdirectory if provided.
///
/// If a subdirectory name is given, it will be appended to the root path.
/// Otherwise, this function simply returns the root data directory.  
///
/// This is a convenient helper when dealing with optional path scopes.
///
/// The returned path is not guaranteed to exist.
pub async fn join_dir(sub: Option<&str>) -> PathBuf {
  if let Some(sub) = sub {
    return sub_data_dir(sub).await;
  }
  root_data_dir().await
}

/// Returns (and ensures) a temporary data directory for transient files.
///
/// This directory can be used for short-lived data such as cache files,
/// in-progress downloads, or temporary computation results.  
///
/// The directory will be created automatically if it doesn’t already exist.
///
/// # Errors
/// Returns an error if the temporary directory cannot be created.
pub async fn temp_dir() -> anyhow::Result<PathBuf> {
  ensure_exists(sub_data_dir("temp").await).await
}
