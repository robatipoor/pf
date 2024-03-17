use crate::helper::ApiTestContext;
use test_context::test_context;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_index_page(ctx: &mut ApiTestContext) {
  let (status, html) = ctx.index_page().await.unwrap();
  assert!(html.contains(&ctx.state.config.server.get_domain_name()));
  assert!(status.is_success(), "status: {status}");
}
