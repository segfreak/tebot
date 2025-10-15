pub struct Command {
  pub prefix: char,
  pub name: String,
  pub args: Vec<String>,
}

impl Command {
  pub fn with_prefix(s: &str, prefix: char) -> Option<Self> {
    let mut chars = s.chars().peekable();
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    if chars.peek() != Some(&prefix) {
      return None;
    }
    chars.next();

    while let Some(&c) = chars.peek() {
      match c {
        '"' => {
          in_quotes = !in_quotes;
          chars.next();
        }
        ' ' if !in_quotes => {
          if !current.is_empty() {
            parts.push(current.clone());
            current.clear();
          }
          chars.next();
        }
        _ => {
          current.push(c);
          chars.next();
        }
      }
    }

    if !current.is_empty() {
      parts.push(current);
    }

    let name = parts.get(0)?.clone();
    let args = parts.into_iter().skip(1).collect();

    Some(Self {
      prefix: prefix,
      name: name,
      args: args,
    })
  }

  pub fn with_prefixes<T>(s: &str, allowed: T) -> Option<Self>
  where
    T: AsRef<[char]>,
  {
    let mut chars = s.chars();
    let first = chars.next()?;

    if allowed.as_ref().contains(&first) {
      Self::with_prefix(s, first)
    } else {
      None
    }
  }
}
