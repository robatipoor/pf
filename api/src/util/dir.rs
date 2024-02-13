use std::{ffi::OsString, path::PathBuf};

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
  let mut ancestors = current_path.ancestors();

  while let Some(parent) = ancestors.next() {
    let mut read_dir = std::fs::read_dir(parent)?.into_iter();
    while let Some(dir) = read_dir.next() {
      let dir = dir?;
      if dir.file_name() == OsString::from("Cargo.lock") {
        return Ok(parent.to_path_buf());
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
