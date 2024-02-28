use assert_cmd::Command;

use crate::helper::CliTestContext;

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_upload_and_download_command(ctx: &mut CliTestContext) {
  let (file, expected_content) = ctx.create_dummy_file().await.unwrap();
  let url_path = Command::cargo_bin("cli")
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
  Command::cargo_bin("cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "download",
      "--url-path",
      &url_path.to_string(),
      "--destination",
      destination_dir.to_str().unwrap(),
    ])
    .assert()
    .success();

  let destination_file_path = destination_dir.join(file.file_name().unwrap());
  let actual_content = tokio::fs::read_to_string(destination_file_path)
    .await
    .unwrap();
  assert_eq!(actual_content, expected_content);
}

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_upload_encrypt_and_download_decrypt_command(ctx: &mut CliTestContext) {
  let key_nonce = "12345678901234567890123456789012:1234567890123456789";
  let (file, expected_content) = ctx.create_dummy_file().await.unwrap();
  let url_path = Command::cargo_bin("cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "upload",
      "--source-file",
      file.to_str().unwrap(),
      "--key-nonce",
      key_nonce,
      "--output",
      "url-path",
    ])
    .output()
    .unwrap()
    .stdout;
  let url_path = std::str::from_utf8(&url_path).unwrap().trim();
  let destination_dir = ctx.workspace.join("destination_dir");
  tokio::fs::create_dir_all(&destination_dir).await.unwrap();
  Command::cargo_bin("cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "download",
      "--url-path",
      &url_path.to_string(),
      "--destination",
      destination_dir.to_str().unwrap(),
      "--key-nonce",
      key_nonce,
    ])
    .assert()
    .success();

  let destination_file_path = destination_dir.join(file.file_name().unwrap());
  let actual_content = tokio::fs::read_to_string(destination_file_path)
    .await
    .unwrap();
  assert_eq!(actual_content, expected_content);
}