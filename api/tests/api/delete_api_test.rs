use crate::helper::ApiTestContext;
use crate::{assert_response_err, assert_response_ok};
use fake::{Fake, Faker};
use pf_sdk::dto::{response::BodyResponseError, FileUrlPath};
use test_context::test_context;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_delete_exist_file(ctx: &mut ApiTestContext) {
  let file = ctx
    .upload_dummy_file(None, None, None, None, None, None)
    .await;
  let (status, resp) = ctx.info(&file.url_path, None).await.unwrap();
  assert_response_ok!(resp);
  assert!(status.is_success(), "status: {status}");
  ctx.delete(&file.url_path, None).await.unwrap();
  let (status, resp) = ctx.info(&file.url_path, None).await.unwrap();
  assert_response_err!(resp, |e: &BodyResponseError| e.error_type == "NOT_FOUND");
  assert_eq!(status, reqwest::StatusCode::NOT_FOUND);
}

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_delete_not_exist_file(ctx: &mut ApiTestContext) {
  let url_path = FileUrlPath {
    code: Faker.fake(),
    file_name: format!("{}.jpg", Faker.fake::<String>()),
  };
  let (status, _) = ctx.delete(&url_path, None).await.unwrap();
  assert!(status.is_success(), "status: {status}");
}

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_delete_file_is_not_deletable(ctx: &mut ApiTestContext) {
  let file = ctx
    .upload_dummy_file(None, None, None, Some(false), None, None)
    .await;
  let (status, _) = ctx.info(&file.url_path, None).await.unwrap();
  assert!(status.is_success(), "status: {status}");
  let (status, resp) = ctx.delete(&file.url_path, None).await.unwrap();
  assert_response_err!(resp, |e: &BodyResponseError| e.error_type
    == "PERMISSION_DENIED");
  assert!(!status.is_success(), "status: {status}");
  let (status, _) = ctx.info(&file.url_path, None).await.unwrap();
  assert!(status.is_success(), "{status}");
}
