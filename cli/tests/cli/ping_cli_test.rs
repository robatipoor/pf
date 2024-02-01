use assert_cmd::Command;

use crate::helper::CliTestContext;

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_ping_command(ctx: &mut CliTestContext) {
  let _out = Command::cargo_bin("cli")
    .unwrap()
    .args(["--server-addr", &ctx.server_addr, "ping"])
    .assert()
    .success()
    .to_string();
}
