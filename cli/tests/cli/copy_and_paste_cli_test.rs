use assert_cmd::Command;

use crate::helper::{generate_random_key_nonce, CliTestContext};

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_copy_and_paste_command(ctx: &mut CliTestContext) {
  let (file, expected_content) = ctx.create_dummy_file().await.unwrap();
  let url_path = Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "copy",
      "--file-name",
      "test.txt",
      "--output",
      "url-path",
    ])
    .pipe_stdin(file)
    .unwrap()
    .output()
    .unwrap()
    .stdout;
  let url_path = std::str::from_utf8(&url_path).unwrap().trim();
  let actual_content = Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "paste",
      "--url-path",
      url_path,
    ])
    .output()
    .unwrap()
    .stdout;
  let actual_content = std::str::from_utf8(&actual_content).unwrap().trim();
  assert_eq!(actual_content, expected_content);
}

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_copy_encrypt_and_paste_decrypt_command(ctx: &mut CliTestContext) {
  let key_nonce = generate_random_key_nonce();
  let (file, expected_content) = ctx.create_dummy_file().await.unwrap();
  let url_path = Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "copy",
      "--file-name",
      "test.txt",
      "--key-nonce",
      &key_nonce,
      "--output",
      "url-path",
    ])
    .pipe_stdin(file)
    .unwrap()
    .output()
    .unwrap()
    .stdout;
  let url_path = std::str::from_utf8(&url_path).unwrap().trim();
  let actual_content = Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "paste",
      "--url-path",
      url_path,
      "--key-nonce",
      &key_nonce,
    ])
    .output()
    .unwrap()
    .stdout;
  let actual_content = std::str::from_utf8(&actual_content).unwrap().trim();
  assert_eq!(actual_content, expected_content);
}
