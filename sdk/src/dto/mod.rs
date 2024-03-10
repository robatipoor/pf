use crate::util::url::create_url;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod request;
pub mod response;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, fake::Dummy)]
pub struct FileUrlPath {
  pub code: String,
  pub file_name: String,
}

impl FileUrlPath {
  pub fn from_url(url: &str) -> anyhow::Result<Self> {
    let url_path = url::Url::parse(url)?.path().to_string();
    Self::from_str(&url_path)
  }

  pub fn to_url(&self, base_url: &str) -> Result<url::Url, url::ParseError> {
    create_url(base_url, &self.code, &self.file_name)
  }
}

impl std::fmt::Display for FileUrlPath {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}/{}", self.code, self.file_name)
  }
}

impl FromStr for FileUrlPath {
  type Err = anyhow::Error;

  fn from_str(input: &str) -> Result<Self, Self::Err> {
    let input = input.trim_start_matches('/').split('/').collect::<Vec<_>>();

    if input.len() != 2 {
      return Err(anyhow::anyhow!("The file url path is invalid."));
    }

    let code = input[0].to_string();
    let file_name = input[1].to_string();

    Ok(Self { code, file_name })
  }
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_from_str_file_url_path() {
    FileUrlPath::from_str("/ABCD/filename").unwrap();
    FileUrlPath::from_str("ABCD/filename.txt").unwrap();
    assert!(FileUrlPath::from_str("//file").is_err());
    assert!(FileUrlPath::from_str("file").is_err());
    assert!(FileUrlPath::from_str("A/B/file").is_err());
  }
}
