// use test_context::test_context;
//
// use crate::helper::ApiTestContext;
//
// #[test_context(ApiTestContext)]
// #[tokio::test]
// pub async fn test_(ctx: &mut ApiTestContext) {
//   let (status, body) = ctx.health_check().await.unwrap();
//   assert!(body.is_ok());
//   assert!(status.is_success());
// }
