use test_context::test_context;

use crate::helper::ApiTestContext;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_home_page(ctx: &mut ApiTestContext) {
  let (status, body) = ctx.home_page().await.unwrap();
  assert!(!body.is_empty(), "body: {body}");
  assert!(status.is_success(), "status: {status}");
}
