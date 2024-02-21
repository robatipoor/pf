use std::path::{Path, PathBuf};

use anyhow::anyhow;

pub fn get_file_name(path: &Path) -> anyhow::Result<String> {
  path
    .file_name()
    .and_then(|n| n.to_str())
    .map(|n| n.to_string())
    .ok_or_else(|| anyhow!("The source path must include the file name."))
}

pub fn get_content_type(path: &Path) -> anyhow::Result<String> {
  mime_guess::from_path(path)
    .first()
    .map(|meme| meme.essence_str().to_owned())
    .ok_or_else(|| anyhow!("The source file name must include the extension."))
}

pub fn add_extension(path: impl AsRef<Path>, extension: &str) -> PathBuf {
  let mut result = path.as_ref().to_owned();

  result.set_extension(
    result
      .extension()
      .and_then(|ext| ext.to_str())
      .map(|ext| format!("{ext}.{extension}"))
      .unwrap_or_else(|| extension.to_string()),
  );

  result
}

pub fn add_parent_dir(file: impl AsRef<Path>, path: &str) -> anyhow::Result<PathBuf> {
  let file = file.as_ref().to_owned();
  let file_name = file
    .file_name()
    .ok_or_else(|| anyhow::anyhow!("Failed to get file name"))?;
  let parent_dir = file
    .parent()
    .map_or_else(|| PathBuf::from(path), |p| p.join(path));

  Ok(parent_dir.join(file_name))
}

pub fn rm_extra_extension(path: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
  let mut result = path.as_ref().to_owned();
  let file_stem = result
    .file_stem()
    .ok_or_else(|| anyhow::anyhow!("Failed to get file stem"))?;
  if let Some(parent) = result.parent() {
    result = parent.join(file_stem);
  } else {
    result = PathBuf::from(file_stem);
  }

  Ok(result)
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_get_file_name() {
    let result = get_file_name(Path::new("/test/file.txt")).unwrap();
    assert_eq!(result, "file.txt");
  }

  #[test]
  fn test_get_content_type() {
    let result = get_content_type(Path::new("/test/file.txt")).unwrap();
    assert_eq!(result, "text/plain");
    let result = get_content_type(Path::new("/test/file.txt.bin")).unwrap();
    assert_eq!(result, "application/octet-stream");
  }

  #[test]
  fn test_add_extension() {
    let result = add_extension(Path::new("/test/file.txt"), "enc");
    assert_eq!(result, PathBuf::from("/test/file.txt.enc"));
    let result = add_extension(Path::new("/test/file"), "enc");
    assert_eq!(result, PathBuf::from("/test/file.enc"))
  }

  #[test]
  fn test_rm_extra_extension() {
    let result = rm_extra_extension(Path::new("/test/file.txt.ext")).unwrap();
    assert_eq!(result, PathBuf::from("/test/file.txt"));
    let result = rm_extra_extension(Path::new("/test/file")).unwrap();
    assert_eq!(result, PathBuf::from("/test/file"));
    let result = rm_extra_extension(Path::new("file")).unwrap();
    assert_eq!(result, PathBuf::from("file"));
  }

  #[test]
  fn test_add_parent_dir() {
    let result = add_parent_dir(Path::new("/test/file.txt.ext"), "test2").unwrap();
    assert_eq!(result, PathBuf::from("/test/test2/file.txt.ext"));
    let result = add_parent_dir(Path::new("/test/file"), "test2").unwrap();
    assert_eq!(result, PathBuf::from("/test/test2/file"));
    let result = add_parent_dir(Path::new("file"), "test2").unwrap();
    assert_eq!(result, PathBuf::from("test2/file"));
  }
}
