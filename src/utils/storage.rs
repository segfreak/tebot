use std::path::PathBuf;

use tokio::io::AsyncWriteExt;

use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::*;
use teloxide::Bot;

pub async fn download_file(
  bot: &Bot,
  file_id: FileId,
  path: &PathBuf,
) -> anyhow::Result<()> {
  let file = bot.get_file(file_id).await?;
  let mut out_file = tokio::fs::File::create(path).await?;
  bot.download_file(&file.path, &mut out_file).await?;
  out_file.flush().await?;
  Ok(())
}
