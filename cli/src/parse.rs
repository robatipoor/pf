use std::{path::PathBuf, str::FromStr};

use anyhow::anyhow;
use pf_sdk::dto::FileUrlPath;

use pf_sdk::util::crypto::{KeyNonce, KeyType, NonceType};

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
    "second" | "sec" | "s" => 1,
    "minute" | "min" => 60,
    "hour" | "h" => 3600,
    "day" | "d" => 3600 * 24,
    "week" | "w" => 3600 * 24 * 7,
    "month" => 3600 * 24 * 30,
    "year" | "y" => 3600 * 24 * 30 * 12,
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

pub fn parse_file_name(file_name: &str) -> anyhow::Result<PathBuf> {
  let source_file = PathBuf::from(file_name);
  if source_file.extension().is_none() {
    return Err(anyhow!("The file name should include an extension."));
  }
  Ok(source_file)
}

pub fn parse_destination(destination: &str) -> anyhow::Result<PathBuf> {
  let destination = PathBuf::from(destination);
  if let Some(file_name) = destination.file_name() {
    match destination.parent() {
      Some(parent) if !parent.as_os_str().is_empty() => {
        return Ok(parent.canonicalize()?.join(file_name));
      }
      _ => {
        return Ok(destination);
      }
    }
  }
  Ok(destination.canonicalize()?)
}

pub fn parse_file_url_path(file_path: &str) -> anyhow::Result<FileUrlPath> {
  FileUrlPath::from_str(file_path)
}

#[cfg(test)]
mod tests {

  use super::*;
  use fake::{Fake, Faker};
  use pf_sdk::{assert_err, util::random::generate_random_string};

  #[test]
  fn test_parse_key_nonce() {
    let key = generate_random_string(32);
    let nonce = generate_random_string(19);
    parse_key_nonce(&format!("{key}:{nonce}")).unwrap();
    let result = parse_key_nonce("key:nonce");
    assert_err!(result);
  }

  #[test]
  fn test_parse_auth() {
    let username: String = Faker.fake();
    let password: String = Faker.fake();
    let (actual_user, actual_pass) = parse_auth(&format!("{username}:{password}")).unwrap();
    assert_eq!(username, actual_user);
    assert_eq!(password, actual_pass);
    let result = parse_auth("test");
    assert_err!(result);
  }

  #[test]
  fn test_parse_expire_time() {
    assert_eq!(parse_expire_time("1 s").unwrap(), 1);
    assert_eq!(parse_expire_time("10 sec").unwrap(), 10);
    assert_eq!(parse_expire_time("10 min").unwrap(), 600);
    assert_eq!(parse_expire_time("10 day").unwrap(), 864000);
    assert_eq!(parse_expire_time("1 week").unwrap(), 604800);
    assert_eq!(parse_expire_time("1 hour").unwrap(), 3600);
    assert_eq!(parse_expire_time("1 y").unwrap(), 31104000);
    let result = parse_expire_time("125y");
    assert_err!(result);
  }

  #[test]
  fn test_parse_source_file() {
    let result = parse_source_file("foo/bar/file.txt");
    assert_err!(result);
  }

  #[test]
  fn test_parse_destination() {
    let result = parse_destination("file_name.txt").unwrap();
    assert_eq!(PathBuf::from("file_name.txt"), result);
    let result = parse_destination("//file_name.txt").unwrap();
    assert_eq!(PathBuf::from("/file_name.txt"), result);
    let result = parse_destination("./file_name.txt").unwrap();
    assert_eq!(
      std::env::current_dir().unwrap().join("file_name.txt"),
      result
    );
    let result = parse_destination("/foo/bar/test.txt");
    assert_err!(result);
  }
}
