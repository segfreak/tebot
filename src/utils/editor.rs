//! Editor detection and management utilities.
//!
//! This module provides a cross-platform abstraction for working with
//! text editors, allowing automatic detection, listing, and opening of
//! editors in both blocking and non-blocking modes.

use std::process::Stdio;
use std::{env, path::PathBuf, process::Command};

/// Represents a system-installed text editor.
///
/// Stores both the editor's name and the absolute path to its executable.
/// Can be used to open files or run editor commands directly from code.
#[derive(Debug, Clone)]
pub struct Editor {
  /// Editor display name (e.g., "vim" or "code.exe").
  pub name: String,
  /// Path to the editor executable on the filesystem.
  pub path: PathBuf,
}

impl Editor {
  /// Creates a new [`Editor`] instance from name and path.
  ///
  /// This method does not validate whether the provided path points to
  /// an existing executable. It simply wraps the given values into
  /// a convenient structure for later use with `open` or `open_file`.
  ///
  /// Typically used internally when discovering editors on the system.
  pub fn new(
    name: impl Into<String>,
    path: impl Into<PathBuf>,
  ) -> Self {
    Self {
      name: name.into(),
      path: path.into(),
    }
  }

  /// Returns a list of known editor executable names.
  ///
  /// This includes popular editors like `vim`, `nvim`, `nano`, and `code`.
  /// On Windows, `.exe` variants such as `notepad.exe` are used instead.
  ///
  /// The returned list represents potential candidates to check in PATH.
  /// It does not guarantee that any of them actually exist on the system.
  pub fn candidates() -> Vec<&'static str> {
    #[cfg(target_os = "windows")]
    let _candidates = vec!["notepad.exe", "notepad++.exe", "code.exe"];

    #[cfg(not(target_os = "windows"))]
    let _candidates = vec![
      "lvim", "nvim", "vim", "code", "nano", "micro", "emacs", "vi",
    ];

    _candidates
  }

  /// Detects and returns the first available editor on the system.
  ///
  /// It first checks the `EDITOR` environment variable (on non-Windows),
  /// then scans through all known candidates listed in [`candidates`].
  ///
  /// If no matching executable is found, this function fails with an error.
  /// This is the primary entry point for automatically selecting an editor.
  pub fn detect() -> anyhow::Result<Self> {
    #[cfg(not(target_os = "windows"))]
    if let Ok(editor) = env::var("EDITOR") {
      if let Some(path) = which::which(&editor).ok() {
        return Ok(Self::new(editor, path));
      }
    }

    if let Some(found) = Self::list().into_iter().next() {
      return Ok(found);
    }

    anyhow::bail!("no suitable text editor found");
  }

  /// Lists all editors available in the system PATH.
  ///
  /// Each entry in the returned vector represents an installed editor
  /// whose executable was successfully located using the `which` utility.
  ///
  /// The result may be empty if no editors were found in PATH.
  /// The output list is deduplicated by name but not guaranteed to be sorted.
  pub fn list() -> Vec<Self> {
    let mut editors = Vec::new();

    for cand in Self::candidates() {
      if let Ok(path) = which::which(cand) {
        editors.push(Self::new(cand, path));
      }
    }

    editors.dedup_by(|a, b| a.name == b.name);
    editors
  }

  fn block(
    &self,
    cmd: &mut Command,
  ) {
    if self.name.contains("code") {
      cmd.arg("--wait");
    }
  }

  /// Opens a file in the selected editor.
  ///
  /// The `path` argument specifies which file to open.
  /// The `blocking` flag controls whether the call waits for the editor
  /// to close before returning (true) or runs asynchronously (false).
  ///
  /// This function automatically applies editor-specific flags such as
  /// `--wait` for Visual Studio Code. Supports Windows, macOS, and Linux.
  pub fn open_file(
    &self,
    path: impl Into<PathBuf>,
    blocking: bool,
  ) -> anyhow::Result<()> {
    let path = path.into();

    #[cfg(target_os = "windows")]
    {
      let mut cmd = Command::new(&self.path);
      cmd.arg(&path);

      if blocking {
        self.block(&mut cmd);
        cmd.status()?;
      } else {
        cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
      }
    }

    #[cfg(target_os = "linux")]
    {
      let mut cmd = Command::new(&self.path);
      cmd.arg(&path);

      if blocking {
        self.block(&mut cmd);
        cmd.status()?;
      } else {
        cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
      }
    }

    #[cfg(target_os = "macos")]
    {
      let mut cmd = Command::new("open");
      cmd.arg("-a").arg(&self.path);
      cmd.arg(&path);

      if blocking {
        self.block(&mut cmd);
        cmd.arg("-W").status()?;
      } else {
        cmd.stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
      }
    }

    Ok(())
  }

  /// Opens the editor with custom command-line arguments.
  ///
  /// This allows launching an editor instance for advanced use cases,
  /// such as editing multiple files, specifying configuration flags,
  /// or starting in a specific mode.
  ///
  /// The function waits until the editor process exits before returning.
  /// For asynchronous use, prefer spawning the process manually.
  pub fn open(
    &self,
    args: impl IntoIterator<Item = impl Into<String>>,
  ) -> anyhow::Result<()> {
    let args: Vec<String> = args.into_iter().map(Into::into).collect();

    #[cfg(target_os = "windows")]
    {
      let mut cmd = Command::new(&self.path);
      cmd.args(&args);
      cmd.status()?;
    }

    #[cfg(target_os = "linux")]
    {
      let mut cmd = Command::new(self.path.to_string_lossy().into_owned());
      cmd.args(&args);
      cmd.status()?;
    }

    #[cfg(target_os = "macos")]
    {
      let mut cmd = Command::new("open");
      cmd.arg("-W");
      cmd.arg("-a").arg(&self.path);
      cmd.args(&args);
      cmd.status()?;
    }

    Ok(())
  }
}
