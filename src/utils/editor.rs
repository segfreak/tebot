use std::process::Stdio;
use std::{env, path::PathBuf, process::Command};

#[derive(Debug, Clone)]
pub struct Editor {
  pub name: String,
  pub path: PathBuf,
}

impl Editor {
  pub fn new(
    name: impl Into<String>,
    path: impl Into<PathBuf>,
  ) -> Self {
    Self {
      name: name.into(),
      path: path.into(),
    }
  }

  pub fn candidates() -> Vec<&'static str> {
    #[cfg(target_os = "windows")]
    let _candidates = vec!["notepad.exe", "notepad++.exe", "code.exe"];

    #[cfg(not(target_os = "windows"))]
    let _candidates = vec![
      "lvim", "nvim", "vim", "code", "nano", "micro", "emacs", "vi",
    ];

    _candidates
  }

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

  pub fn list() -> Vec<Self> {
    let mut editors = Vec::new();

    for cand in Self::candidates() {
      if let Ok(path) = which::which(cand) {
        editors.push(Self::new(cand, path));
      }
    }

    // editors.sort_by(|a, b| a.name.cmp(&b.name));
    editors.dedup_by(|a, b| a.name == b.name);

    editors
  }

  fn block(
    &self,
    cmd: &mut Command,
  ) {
    match self.name.as_str() {
      n if n.contains("code") => {
        cmd.arg("--wait");
      }
      _ => {}
    }
  }

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
