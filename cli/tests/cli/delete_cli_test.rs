
#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_delete_command(ctx: &mut CliTestContext) {
  let code: String = Faker.fake();
  let file_name: String = Faker.fake();
  // ctx.mock_delete_api(&code, &file_name).await;
  let _out = Command::cargo_bin("cli")
    .unwrap()
    .args([
      "--url",
      &format!("{}/{code}/{file_name}", &ctx.server.uri()),
      "delete",
    ])
    .assert()
    .success()
    .to_string();
}