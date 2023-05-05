use crate::helper::ApiTestContext;
use common::unwrap;
use test_context::test_context;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_info(ctx: &mut ApiTestContext) {
  let file = ctx.upload_dummy_file(None, None, None, None).await;
  let (status, resp) = ctx.info(&file.path, None).await.unwrap();
  let resp = unwrap!(resp);
  assert!(status.is_success(), "status: {status}");
  assert!(resp.is_deleteable);
}
