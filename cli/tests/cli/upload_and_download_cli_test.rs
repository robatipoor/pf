use assert_cmd::Command;
use pf_sdk::util::file::add_extension;

use crate::helper::{generate_random_key_nonce, CliTestContext};

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_upload_and_download_command_with_progress_bar(ctx: &mut CliTestContext) {
  let (file, expected_content) = ctx.create_dummy_file().await.unwrap();
  let url_path = Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "upload",
      "--source-file",
      file.to_str().unwrap(),
      "--progress-bar",
      "--output",
      "url-path",
    ])
    .output()
    .unwrap()
    .stdout;
  let url_path = std::str::from_utf8(&url_path).unwrap().trim();
  let destination_dir = ctx.workspace.join("destination_dir");
  tokio::fs::create_dir_all(&destination_dir).await.unwrap();
  Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "download",
      "--url-path",
      url_path,
      "--destination",
      destination_dir.to_str().unwrap(),
      "--progress-bar",
    ])
    .assert()
    .success();

  let destination_file_path = destination_dir.join(file.file_name().unwrap());
  let actual_content = tokio::fs::read_to_string(&destination_file_path)
    .await
    .unwrap();
  assert_eq!(actual_content, expected_content);
}

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_upload_and_download_command_to_the_destination_dir(ctx: &mut CliTestContext) {
  let (file, expected_content) = ctx.create_dummy_file().await.unwrap();
  let url_path = Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "upload",
      "--source-file",
      file.to_str().unwrap(),
      "--output",
      "url-path",
    ])
    .output()
    .unwrap()
    .stdout;
  let url_path = std::str::from_utf8(&url_path).unwrap().trim();
  let destination_dir = ctx.workspace.join("destination_dir");
  tokio::fs::create_dir_all(&destination_dir).await.unwrap();
  Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "download",
      "--url-path",
      url_path,
      "--destination",
      destination_dir.to_str().unwrap(),
    ])
    .assert()
    .success();

  let destination_file_path = destination_dir.join(file.file_name().unwrap());
  let actual_content = tokio::fs::read_to_string(&destination_file_path)
    .await
    .unwrap();
  assert_eq!(actual_content, expected_content);
}

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_upload_encrypt_and_download_decrypt_command_to_the_destination_dir(
  ctx: &mut CliTestContext,
) {
  let key_nonce = generate_random_key_nonce();
  let (file, expected_content) = ctx.create_dummy_file().await.unwrap();
  let url_path = Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "upload",
      "--source-file",
      file.to_str().unwrap(),
      "--key-nonce",
      &key_nonce,
      "--output",
      "url-path",
    ])
    .output()
    .unwrap()
    .stdout;
  let encrypt_file = format!("{}.bin", file.to_str().unwrap());
  assert!(!tokio::fs::try_exists(encrypt_file).await.unwrap());
  let url_path = std::str::from_utf8(&url_path).unwrap().trim();

  let destination_dir = ctx.workspace.join("destination_dir");
  tokio::fs::create_dir_all(&destination_dir).await.unwrap();
  Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "download",
      "--url-path",
      url_path,
      "--destination",
      destination_dir.to_str().unwrap(),
      "--key-nonce",
      &key_nonce,
    ])
    .assert()
    .success();
  let encrypt_file = destination_dir.join(format!(
    "{}.bin",
    file.file_name().unwrap().to_str().unwrap(),
  ));
  assert!(!tokio::fs::try_exists(encrypt_file).await.unwrap());
  let destination_file_path = destination_dir.join(file.file_name().unwrap());
  let actual_content = tokio::fs::read_to_string(destination_file_path)
    .await
    .unwrap();
  assert_eq!(actual_content, expected_content);
}

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_upload_and_download_command_to_the_destination_file(ctx: &mut CliTestContext) {
  let (url_path, expected_content) = ctx.upload_dummy_file().await.unwrap();
  let destination_file_path = ctx.workspace.join("destination_file.text");
  Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "download",
      "--url-path",
      &url_path.to_string(),
      "--destination",
      destination_file_path.to_str().unwrap(),
      "--progress-bar",
    ])
    .assert()
    .success();
  let actual_content = tokio::fs::read_to_string(&destination_file_path)
    .await
    .unwrap();
  assert_eq!(actual_content, expected_content);
}

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_upload_encrypt_and_download_decrypt_command_to_the_destination_file(
  ctx: &mut CliTestContext,
) {
  let key_nonce = generate_random_key_nonce();
  let (file, expected_content) = ctx.create_dummy_file().await.unwrap();
  let url_path = Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "upload",
      "--source-file",
      file.to_str().unwrap(),
      "--key-nonce",
      &key_nonce,
      "--output",
      "url-path",
    ])
    .output()
    .unwrap()
    .stdout;
  let encrypt_file = format!("{}.bin", file.to_str().unwrap());
  assert!(!tokio::fs::try_exists(encrypt_file).await.unwrap());
  let url_path = std::str::from_utf8(&url_path).unwrap().trim();
  let destination_file_path = ctx.workspace.join("destination_file.txt");
  Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "download",
      "--url-path",
      url_path,
      "--destination",
      destination_file_path.to_str().unwrap(),
      "--key-nonce",
      &key_nonce,
    ])
    .assert()
    .success();
  let encrypt_file = add_extension(&destination_file_path, "bin");
  assert!(!tokio::fs::try_exists(encrypt_file).await.unwrap());
  let actual_content = tokio::fs::read_to_string(destination_file_path)
    .await
    .unwrap();
  assert_eq!(actual_content, expected_content);
}
