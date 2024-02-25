use assert_cmd::Command;

use crate::helper::CliTestContext;

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_upload_command(ctx: &mut CliTestContext) {
  let (file, content) = ctx.create_dummy_file().await.unwrap();
  let key_nonce = "12345678901234567890123456789012:1234567890123456789";
  let _out = Command::cargo_bin("cli")
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
      key_nonce,
    ])
    .assert()
    .success()
    .to_string();
  let destination = ctx.workspace.join("destination_file.txt");
  let _out = Command::cargo_bin("cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "decrypt",
      "--source-file",
      &format!("{}.bin", file.to_str().unwrap()),
      "--destination",
      destination.to_str().unwrap(),
      "--key-nonce",
      key_nonce,
    ])
    .assert()
    .success()
    .to_string();

  let actual_content = tokio::fs::read_to_string(destination).await.unwrap();
  assert_eq!(actual_content, content);
}
