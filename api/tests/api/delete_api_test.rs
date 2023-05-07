use crate::helper::ApiTestContext;
use common::{assert_err, assert_ok, error::BodyResponseError};
use fake::{Fake, Faker};
use hyper::StatusCode;
use test_context::test_context;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_delete_exist_file(ctx: &mut ApiTestContext) {
  let file = ctx.upload_dummy_file(None, None, None, None, None).await;
  let (status, resp) = ctx.info(&file.path, None).await.unwrap();
  assert_ok!(resp);
  assert!(status.is_success(), "status: {status}");
  ctx.delete(&file.path, None).await.unwrap();
  let (status, resp) = ctx.info(&file.path, None).await.unwrap();
  assert_err!(resp, |e: &BodyResponseError| e.error_type == "NOT_FOUND");
  assert_eq!(status, StatusCode::NOT_FOUND);
}

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_delete_not_exist_file(ctx: &mut ApiTestContext) {
  let path = format!("{}/{}.jpg", Faker.fake::<String>(), Faker.fake::<String>());
  let (status, _) = ctx.delete(&path, None).await.unwrap();
  assert!(status.is_success(), "status: {status}");
}

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_delete_file_is_not_deleteable(ctx: &mut ApiTestContext) {
  let file = ctx
    .upload_dummy_file(None, None, None, Some(false), None)
    .await;
  let (status, _) = ctx.info(&file.path, None).await.unwrap();
  assert!(status.is_success(), "status: {status}");
  let (status, resp) = ctx.delete(&file.path, None).await.unwrap();
  assert_err!(resp, |e: &BodyResponseError| e.error_type
    == "PERMISSION_DENIED");
  assert!(!status.is_success(), "status: {status}");
  let (status, _) = ctx.info(&file.path, None).await.unwrap();
  assert!(status.is_success(), "{status}");
}
