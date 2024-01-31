use std::path::Path;

use anyhow::anyhow;

pub fn get_file_name(source: &Path) -> anyhow::Result<String> {
  source
    .file_name()
    .map(|n| n.to_str().map(|n| n.to_string()))
    .flatten()
    .ok_or_else(|| anyhow!("The source path must include the file name."))
}

pub fn get_content_type(source: &Path) -> anyhow::Result<String> {
  mime_guess::from_path(source)
    .first()
    .map(|mem| mem.essence_str().to_owned())
    .ok_or_else(|| anyhow!("The source file name must include the extension."))
}
