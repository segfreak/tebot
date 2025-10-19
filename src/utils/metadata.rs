use std::env;

use std::fmt;

impl fmt::Display for Package {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    write!(
      f,
      "{} v{}.{}.{}",
      self.name, self.version.major, self.version.minor, self.version.patch,
    )
  }
}

impl fmt::Display for GitMetadata {
  fn fmt(
    &self,
    f: &mut fmt::Formatter<'_>,
  ) -> fmt::Result {
    let short_commit = &self.commit[..self.commit.len().min(7)];
    let dirty_mark = if self.dirty { "*" } else { "" };
    if !self.tag.is_empty() {
      write!(
        f,
        "{}@{}{} ({})",
        self.branch, short_commit, dirty_mark, self.tag
      )
    } else {
      write!(f, "{}@{}{}", self.branch, short_commit, dirty_mark)
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Version {
  pub major: u64,
  pub minor: u64,
  pub patch: u64,
}

impl Version {
  pub fn from_str(s: &str) -> anyhow::Result<Self> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
      return Err(anyhow::anyhow!("invalid version string: '{}'", s));
    }

    let major = parts[0]
      .parse::<u64>()
      .map_err(|e| anyhow::anyhow!("failed to parse major version: {}", e))?;
    let minor = parts[1]
      .parse::<u64>()
      .map_err(|e| anyhow::anyhow!("failed to parse minor version: {}", e))?;
    let patch = parts[2]
      .parse::<u64>()
      .map_err(|e| anyhow::anyhow!("failed to parse patch version: {}", e))?;

    Ok(Self {
      major,
      minor,
      patch,
    })
  }
}

#[derive(Debug, Clone)]
pub struct Package {
  pub name: String,
  pub version: Version,
  pub authors: Vec<String>,
  pub repo: String,
}

impl Package {
  pub fn from_env() -> anyhow::Result<Self> {
    let name =
      env::var("CARGO_PKG_NAME").map_err(|_| anyhow::anyhow!("CARGO_PKG_NAME is not set"))?;

    let ver_str =
      env::var("CARGO_PKG_VERSION").map_err(|_| anyhow::anyhow!("CARGO_PKG_VERSION is not set"))?;
    let version = Version::from_str(&ver_str)?;

    let authors_str = env::var("CARGO_PKG_AUTHORS").unwrap_or_default();
    let authors = authors_str.split(':').map(|s| s.to_string()).collect();

    let repo = env::var("CARGO_PKG_REPOSITORY").unwrap_or("unknown".to_string());

    Ok(Self {
      name,
      version,
      authors,
      repo,
    })
  }
}

#[derive(Debug)]
pub struct GitMetadata {
  pub branch: String,
  pub commit: String,
  pub dirty: bool,
  pub tag: String,
  pub repo: String,
}

impl GitMetadata {
  pub fn from_env() -> anyhow::Result<Self> {
    let branch = env::var("GIT_BRANCH").map_err(|_| anyhow::anyhow!("GIT_BRANCH is not set"))?;
    let commit = env::var("GIT_COMMIT").map_err(|_| anyhow::anyhow!("GIT_COMMIT is not set"))?;
    let dirty_str = env::var("GIT_DIRTY").map_err(|_| anyhow::anyhow!("GIT_DIRTY is not set"))?;
    let tag = env::var("GIT_TAG").unwrap_or_default();

    let dirty = match dirty_str.as_str() {
      "0" => false,
      "1" => true,
      other => {
        return Err(anyhow::anyhow!(
          "invalid value for GIT_DIRTY: '{}', expected '0' or '1'",
          other
        ))
      }
    };

    let repo = std::env::var("GIT_REPOSITORY").unwrap_or_else(|_| "unknown".into());

    Ok(Self {
      branch,
      commit,
      dirty,
      tag,
      repo,
    })
  }
}
