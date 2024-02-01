use assert_cmd::Command;

use crate::helper::CliTestContext;

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_upload_command(ctx: &mut CliTestContext) {
  let (file, _) = ctx.create_dummy_file().await.unwrap();
  let _out = Command::cargo_bin("cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "upload",
      "--source-file",
      file.to_str().unwrap(),
    ])
    .assert()
    .success()
    .to_string();
}
