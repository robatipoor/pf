use test_context::test_context;

use crate::helper::ApiTestContext;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_download(ctx: &mut ApiTestContext) {
  let path = ctx.upload_dummy_object(None, None, None, None).await;
  let (status, body) = ctx.download(&path, None).await.unwrap();
  assert!(status.is_success());
  // assert_eq!(file, body);
}
