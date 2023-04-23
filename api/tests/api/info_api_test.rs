use crate::helper::ApiTestContext;
use test_context::test_context;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_info(ctx: &mut ApiTestContext) {
  let file = ctx.upload_dummy_file(None, None, None, None).await;
  let (status, resp) = ctx.info(&file.path, None).await.unwrap();
  assert!(status.is_success());
  let resp = resp.unwrap();
  assert!(resp.is_deleteable);
}
