use crate::helper::ApiTestContext;
use test_context::test_context;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_delete(ctx: &mut ApiTestContext) {
  let path = ctx.upload_dummy_object(None, None, None, None).await;
  // let (status, body) = ctx.download(&url.path()[1..], None).await.unwrap();
  // assert!(status.is_success());
}
