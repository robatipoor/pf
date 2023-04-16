use crate::helper::ApiTestContext;
use test_context::test_context;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_info(ctx: &mut ApiTestContext) {
  let path = ctx.upload_dummy_object(None, None, None, None).await;
  let (status, resp) = ctx.info(&path, None).await.unwrap();
  assert!(status.is_success());
  let resp = resp.unwrap();
  // assert_eq!(resp.fi);
}
