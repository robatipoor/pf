use std::{path::PathBuf, str::FromStr};

use anyhow::anyhow;
use sdk::dto::FileUrlPath;

use crate::util::crypto::{KeyNonce, KeyType, NonceType};

pub fn parse_key_nonce(input: &str) -> anyhow::Result<KeyNonce> {
  let pos = input
    .find(':')
    .ok_or_else(|| anyhow!("Invalid key:nonce: no `:` found in {input}"))?;
  let (key_str, nonce_str) = input.split_at(pos);
  if !key_str.is_ascii() || !nonce_str.is_ascii() {
    return Err(anyhow!(
      "ASCII characters should be used for the `key_nonce`."
    ));
  }
  let key = KeyType::new(key_str)?;
  let nonce = NonceType::new(&nonce_str[1..])?;
  Ok(KeyNonce { key, nonce })
}

pub fn parse_auth(input: &str) -> anyhow::Result<(String, String)> {
  let pos = input
    .find(':')
    .ok_or_else(|| anyhow!("Invalid username:password: no `:` found in {input}"))?;
  Ok((input[..pos].parse()?, input[pos + 1..].parse()?))
}

pub fn parse_expire_time(input: &str) -> anyhow::Result<u64> {
  let words: Vec<&str> = input.split_whitespace().collect();
  if words.len() != 2 {
    return Err(anyhow!("Invalid expire time format"));
  }
  let value: u64 = words[0].parse()?;
  let multiplier = match words[1].to_lowercase().as_str() {
    "second" | "sec" | "s" => value,
    "minute" | "min" => value * 60,
    "hour" | "h" => value * 3600,
    "day" | "d" => value * 3600 * 24,
    "month" => value * 3600 * 24 * 30,
    "year" | "y" => value * 3600 * 24 * 30 * 12,
    _ => return Err(anyhow!("Invalid expire time format")),
  };
  Ok(value * multiplier)
}

pub fn parse_source_file(source_file: &str) -> anyhow::Result<PathBuf> {
  let source_file = PathBuf::from(source_file).canonicalize()?;
  if source_file.is_dir() {
    Err(anyhow!(
      "The source file option should be set to the path file."
    ))
  } else {
    Ok(source_file)
  }
}

pub fn parse_destination(destination: &str) -> anyhow::Result<PathBuf> {
  let destination = PathBuf::from(destination).canonicalize()?;
  Ok(destination)
}

pub fn parse_file_url_path(file_path: &str) -> anyhow::Result<FileUrlPath> {
  FileUrlPath::from_str(file_path)
}
