use std::env;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Package {
  pub name: String,
  pub version: Version,
  pub authors: Vec<String>,
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

    Ok(Self {
      name,
      version,
      authors,
    })
  }
}
