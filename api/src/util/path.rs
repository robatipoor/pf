use std::path::{Path, PathBuf};

use crate::database::file_path::FilePath;

pub fn get_fs_path(base_dir: &Path, file_path: &FilePath) -> PathBuf {
  base_dir.join::<PathBuf>(file_path.into())
}
