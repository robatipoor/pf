
#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_download_command(ctx: &mut CliTestContext) {
  let code: String = Faker.fake();
  let file_name = "file.txt";
  // ctx.mock_download_api(&code, &file_name).await;
  let _out = Command::cargo_bin("cli")
    .unwrap()
    .args([
      "--url",
      &format!("{}/{code}/{file_name}", &ctx.server.uri()),
      "download",
      "--path",
      &ctx.download_dir,
    ])
    .assert()
    .success()
    .to_string();
  // TODO try read file
}