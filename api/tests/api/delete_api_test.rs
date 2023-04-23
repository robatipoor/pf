use crate::helper::ApiTestContext;
use fake::{Fake, Faker};
use test_context::test_context;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_delete_exist_file(ctx: &mut ApiTestContext) {
  let file = ctx.upload_dummy_file(None, None, None, None).await;
  let (status, resp) = ctx.info(&file.path, None).await.unwrap();
  assert!(status.is_success());
  let _ = resp.unwrap();
  ctx.delete(&file.path, None).await.unwrap();
  let (status, resp) = ctx.info(&file.path, None).await.unwrap();
  assert!(status.is_success());
  assert!(matches!(resp,common::error::ApiResponseResult::Err(e) if e.error_type == "NOT_FOUND"));
}

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_delete_not_exist_file(ctx: &mut ApiTestContext) {
  let path: String = Faker.fake();
  let (status, resp) = ctx.delete(&path, None).await.unwrap();
  assert!(status.is_success());
  assert!(matches!(resp,common::error::ApiResponseResult::Err(e) if e.error_type == "NOT_FOUND"));
}
