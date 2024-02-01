use assert_cmd::Command;

use crate::helper::CliTestContext;

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_delete_command(ctx: &mut CliTestContext) {
  let (url_path, _) = ctx.upload_dummy_file().await.unwrap();
  let _out = Command::cargo_bin("cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "delete",
      "--url-path",
      &url_path,
    ])
    .assert()
    .success()
    .to_string();
}
