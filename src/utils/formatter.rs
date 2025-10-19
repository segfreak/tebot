use std::time::Duration;

use super::metadata::{GitMetadata, Package, Version};

pub fn format_version(ver: Version) -> String {
  format!("v{}.{}.{}", ver.major, ver.minor, ver.patch)
}

pub fn format_authors(authors: Vec<String>) -> String {
  format!("{:?}", authors)
}

pub fn format_package(pkg: Package) -> String {
  format!(
    "{} {}",
    pkg.name,
    format_version(pkg.version),
    // format_authors(pkg.authors)
  )
}

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
