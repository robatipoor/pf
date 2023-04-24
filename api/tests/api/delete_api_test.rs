use crate::helper::ApiTestContext;
use fake::{Fake, Faker};
use hyper::StatusCode;
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
  assert!(status == StatusCode::NOT_FOUND);
  assert!(matches!(resp,common::error::ApiResponseResult::Err(e) if e.error_type == "NOT_FOUND"));
}

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_delete_not_exist_file(ctx: &mut ApiTestContext) {
  let path = format!("{}/{}.jpg", Faker.fake::<String>(), Faker.fake::<String>());
  let (status, _) = ctx.delete(&path, None).await.unwrap();
  assert!(status.is_success());
}
