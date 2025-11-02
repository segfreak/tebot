use std::path::PathBuf;

use tokio::io::AsyncWriteExt;

use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::*;
use teloxide::Bot;

/// Downloads a file from Telegram servers using the provided [`Bot`] instance.
///
/// This function retrieves file metadata via [`Bot::get_file`], then downloads
/// its contents asynchronously and writes them to the specified local file path.
/// The operation uses Tokio’s asynchronous I/O, ensuring efficient file writes
/// without blocking the runtime.
///
/// This utility is typically used in message handlers for bots that need
/// to store user-uploaded files locally for later processing.
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
