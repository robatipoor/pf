use test_context::test_context;

use crate::helper::ApiTestContext;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_download(ctx: &mut ApiTestContext) {
  let file = ctx.upload_dummy_file(None, None, None, None).await;
  let (status, body) = ctx.download(&file.path, None).await.unwrap();
  assert!(status.is_success());
  assert_eq!(file.content, body);
}
