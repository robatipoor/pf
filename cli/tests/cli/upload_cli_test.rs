
#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_upload_command(ctx: &mut CliTestContext) {
  // ctx.mock_upload_api().await;
  let _out = Command::cargo_bin("cli")
    .unwrap()
    .args([
      "--url",
      &ctx.server.uri(),
      "upload",
      "--path",
      &ctx.upload_file,
    ])
    .assert()
    .success()
    .to_string();
}
