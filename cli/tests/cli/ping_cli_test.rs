use assert_cmd::Command;

use crate::helper::CliTestContext;

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_ping_command(ctx: &mut CliTestContext) {
  Command::cargo_bin("pf-cli")
    .unwrap()
    .args(["--server-addr", &ctx.server_addr, "ping"])
    .assert()
    .stdout("{\"message\":\"Ok\"}\n")
    .success();
}
