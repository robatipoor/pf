use crate::assert_response_ok;
use crate::helper::ApiTestContext;
use test_context::test_context;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_health_check(ctx: &mut ApiTestContext) {
  let (status, body) = ctx.health_check().await.unwrap();
  assert_response_ok!(body);
  assert!(status.is_success(), "status: {status}");
}
