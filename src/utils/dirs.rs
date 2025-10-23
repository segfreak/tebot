use std::path::PathBuf;

use super::env;
use crate::bot::plugin;

pub async fn ensure_exists(dir: PathBuf) -> anyhow::Result<PathBuf> {
  tokio::fs::create_dir_all(&dir).await?;
  Ok(dir)
}

pub async fn root_data_dir() -> PathBuf {
  PathBuf::from(env::get_data_dir().await)
}

pub async fn sub_data_dir(sub: &str) -> PathBuf {
  root_data_dir().await.join(sub)
}

pub async fn plugin_data_dir(plugin: plugin::PluginBox) -> anyhow::Result<PathBuf> {
  ensure_exists(sub_data_dir(plugin.name()).await).await
}

pub async fn join_dir(sub: Option<&str>) -> PathBuf {
  if let Some(sub) = sub {
    return sub_data_dir(sub).await;
  }
  root_data_dir().await
}

pub async fn temp_dir() -> anyhow::Result<PathBuf> {
  ensure_exists(sub_data_dir("temp").await).await
}
