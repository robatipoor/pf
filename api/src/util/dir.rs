use std::path::PathBuf;

pub fn get_settings_dir() -> std::io::Result<std::path::PathBuf> {
  Ok(
    get_project_root()
      .or_else(|_| std::env::current_dir())?
      .join("api")
      .join("settings"),
  )
}

pub fn get_project_root() -> std::io::Result<PathBuf> {
  let current_path = std::env::current_dir()?;

  for ancestor in current_path.ancestors() {
    for dir in std::fs::read_dir(ancestor)? {
      let dir = dir?;
      if dir.file_name() == *"Cargo.lock" {
        return Ok(ancestor.to_path_buf());
      }
    }
  }
  Err(std::io::Error::new(
    std::io::ErrorKind::NotFound,
    "Ran out of places to find Cargo.toml",
  ))
}

#[cfg(test)]
mod tests {
  use super::get_project_root;

  #[test]
  fn test_get_project_root() {
    let root = get_project_root().unwrap();
    assert_eq!(root.file_name().unwrap().to_str().unwrap(), "pf");
  }
}
