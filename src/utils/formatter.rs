use std::time::Duration;

use super::metadata::{GitMetadata, Package, Version};

/// Formats a [`Version`] into a human-readable string.
///
/// Produces output like `"v1.2.3"`.
/// Commonly used for displaying application or dependency versions.
pub fn format_version(ver: Version) -> String {
  format!("v{}.{}.{}", ver.major, ver.minor, ver.patch)
}

/// Formats a list of authors into a compact string representation.
///
/// Currently uses Rust’s debug format (`{:?}`) for simplicity.
/// Useful for debugging or showing author information in logs.
pub fn format_authors(authors: Vec<String>) -> String {
  format!("{:?}", authors)
}

/// Formats package information including its name and version.
///
/// Produces output like `"my_app v1.0.0"`.
/// Intended for CLI banners or diagnostic messages.
pub fn format_package(pkg: Package) -> String {
  format!(
    "{} {}",
    pkg.name,
    format_version(pkg.version),
    // format_authors(pkg.authors)
  )
}

/// Formats Git metadata into a concise string representation.
///
/// Includes branch name, short commit hash, and optional tag.
/// Appends a `*` mark if the working directory is dirty.
/// Example: `"main@a1b2c3d (v1.0.0)"` or `"dev@a1b2c3d*"`
pub fn format_git_metadata(gitmeta: &GitMetadata) -> String {
  let short_commit = &gitmeta.commit[..gitmeta.commit.len().min(7)];

  let dirty_mark = if gitmeta.dirty { "*" } else { "" };

  if !gitmeta.tag.is_empty() {
    format!(
      "{}@{}{} ({})",
      gitmeta.branch, short_commit, dirty_mark, gitmeta.tag
    )
  } else {
    format!("{}@{}{}", gitmeta.branch, short_commit, dirty_mark)
  }
}

/// Formats a [`Duration`] into a readable time string.
///
/// Converts seconds into hours, minutes, and seconds as needed.  
/// Examples: `"3s"`, `"5m 12s"`, `"1h 4m 9s"`.
pub fn format_duration(dur: Duration) -> String {
  let secs = dur.as_secs();
  let hours = secs / 3600;
  let minutes = (secs % 3600) / 60;
  let seconds = secs % 60;

  if hours > 0 {
    format!("{}h {}m {}s", hours, minutes, seconds)
  } else if minutes > 0 {
    format!("{}m {}s", minutes, seconds)
  } else {
    format!("{}s", seconds)
  }
}
