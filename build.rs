use std::process::Command;

fn main() {
  let branch = Command::new("git")
    .args(&["rev-parse", "--abbrev-ref", "HEAD"])
    .output()
    .ok()
    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    .unwrap_or_else(|| "unknown".to_string());

  println!("cargo:rustc-env=GIT_BRANCH={}", branch);

  let commit = Command::new("git")
    .args(&["rev-parse", "HEAD"])
    .output()
    .ok()
    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    .unwrap_or_else(|| "unknown".into());
  println!("cargo:rustc-env=GIT_COMMIT={}", commit);

  let dirty = Command::new("git")
    .args(&["status", "--porcelain"])
    .output()
    .ok()
    .map(|o| if o.stdout.is_empty() { "0" } else { "1" })
    .unwrap_or("0");
  println!("cargo:rustc-env=GIT_DIRTY={}", dirty);

  let tag = Command::new("git")
    .args(&["describe", "--tags", "--abbrev=0"])
    .output()
    .ok()
    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    .unwrap_or_else(|| "".into());
  println!("cargo:rustc-env=GIT_TAG={}", tag);

  let repository = Command::new("git")
    .args(&["config", "--get", "remote.origin.url"])
    .output()
    .ok()
    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    .unwrap_or_else(|| "unknown".into());
  println!("cargo:rustc-env=GIT_REPOSITORY={}", repository);
}
