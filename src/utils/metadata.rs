use std::env;
use std::fmt;

/// Represents a semantic version number with major, minor, and patch components.
///
/// Provides a simple and strict `x.y.z` format parser.
/// Used to represent both package and binary versions.
#[derive(Debug, Clone, Copy)]
pub struct Version {
  pub major: u64,
  pub minor: u64,
  pub patch: u64,
}

impl Version {
  /// Parses a version string (e.g., `"1.2.3"`) into a [`Version`] instance.
  ///
  /// Returns an error if the string does not follow the three-component format.
  /// Useful for loading Cargo metadata or versioned configuration files.
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

/// Holds package information extracted from Cargo build metadata.
///
/// Includes name, version, authors, and repository URL.
/// Typically populated via environment variables set by Cargo.
#[derive(Debug, Clone)]
pub struct Package {
  pub name: String,
  pub version: Version,
  pub authors: Vec<String>,
  pub repo: String,
}

impl Package {
  /// Constructs a [`Package`] from Cargo-provided environment variables.
  ///
  /// Fails if mandatory fields like `CARGO_PKG_NAME` or `CARGO_PKG_VERSION`
  /// are missing. Optional fields such as repository URL may default to `"unknown"`.
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

impl fmt::Display for Package {
  /// Formats the package as `"name vX.Y.Z"`.
  ///
  /// Example: `"tebot v1.3.2"`.
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

/// Represents metadata from a Git repository at build time.
///
/// Includes branch, commit hash, tag, and dirty state.
/// Typically used for embedding build provenance into binaries.
#[derive(Debug)]
pub struct GitMetadata {
  pub branch: String,
  pub commit: String,
  pub dirty: bool,
  pub tag: String,
  pub repo: String,
}

impl GitMetadata {
  /// Constructs [`GitMetadata`] from environment variables.
  ///
  /// Expected variables include `GIT_BRANCH`, `GIT_COMMIT`, and `GIT_DIRTY`.
  /// Returns an error if any required variable is missing or invalid.
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

impl fmt::Display for GitMetadata {
  /// Formats Git metadata as `"branch@commit(tag)"` with an optional dirty marker.
  ///
  /// Example: `"main@a1b2c3d (v1.0.0)"` or `"dev@b3f5a9c*"`.
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
