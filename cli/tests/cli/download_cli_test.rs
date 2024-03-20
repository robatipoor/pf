use assert_cmd::Command;

use crate::helper::CliTestContext;

#[test_context::test_context(CliTestContext)]
#[tokio::test]
async fn test_download_command_to_the_destination_file(ctx: &mut CliTestContext) {
  let (url_path, expected_content) = ctx.upload_dummy_file().await.unwrap();
  let destination_file_path = ctx.workspace.join("destination_file.txt");
  Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "download",
      "--url-path",
      &url_path.to_string(),
      "--destination",
      destination_file_path.to_str().unwrap(),
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
async fn test_download_command_to_the_destination_dir(ctx: &mut CliTestContext) {
  let (url_path, expected_content) = ctx.upload_dummy_file().await.unwrap();
  let destination_dir = ctx.workspace.join("destination_dir");
  tokio::fs::create_dir_all(&destination_dir).await.unwrap();
  Command::cargo_bin("pf-cli")
    .unwrap()
    .args([
      "--server-addr",
      &ctx.server_addr,
      "download",
      "--url-path",
      &url_path.to_string(),
      "--destination",
      destination_dir.to_str().unwrap(),
    ])
    .assert()
    .success();

  let destination_file_path = destination_dir.join(url_path.file_name);
  let actual_content = tokio::fs::read_to_string(destination_file_path)
    .await
    .unwrap();
  assert_eq!(actual_content, expected_content);
}
