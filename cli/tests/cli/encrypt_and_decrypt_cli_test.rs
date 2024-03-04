use crate::helper::{generate_random_key_nonce, CliTestContext};
use assert_cmd::Command;

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_encrypt_and_decrypt_to_the_destination_file(ctx: &mut CliTestContext) {
  let (file, expected_content) = ctx.create_dummy_file().await.unwrap();
  let key_nonce = generate_random_key_nonce();
  Command::cargo_bin("cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "encrypt",
      "--source-file",
      file.to_str().unwrap(),
      "--destination",
      ctx.workspace.to_str().unwrap(),
      "--key-nonce",
      &key_nonce,
    ])
    .assert()
    .success();
  let destination_file_path = ctx.workspace.join("destination_file.txt");
  Command::cargo_bin("cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "decrypt",
      "--source-file",
      &format!("{}.bin", file.to_str().unwrap()),
      "--destination",
      destination_file_path.to_str().unwrap(),
      "--key-nonce",
      &key_nonce,
    ])
    .assert()
    .success();

  let actual_content = tokio::fs::read_to_string(destination_file_path)
    .await
    .unwrap();
  assert_eq!(actual_content, expected_content);
}

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_encrypt_file_and_decrypt_to_the_destination_dir(ctx: &mut CliTestContext) {
  let (file, expected_content) = ctx.create_dummy_file().await.unwrap();
  let key_nonce = generate_random_key_nonce();
  Command::cargo_bin("cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "encrypt",
      "--source-file",
      file.to_str().unwrap(),
      "--destination",
      ctx.workspace.to_str().unwrap(),
      "--key-nonce",
      &key_nonce,
    ])
    .assert()
    .success();
  let destination_dir = ctx.workspace.join("destination_dir");
  tokio::fs::create_dir_all(&destination_dir).await.unwrap();
  Command::cargo_bin("cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "decrypt",
      "--source-file",
      &format!("{}.bin", file.to_str().unwrap()),
      "--destination",
      destination_dir.to_str().unwrap(),
      "--key-nonce",
      &key_nonce,
    ])
    .assert()
    .success();
  let destination_file_path = destination_dir.join(file.file_name().unwrap());
  let actual_content = tokio::fs::read_to_string(destination_file_path)
    .await
    .unwrap();
  assert_eq!(actual_content, expected_content);
}
