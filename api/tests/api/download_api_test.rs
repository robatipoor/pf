use crate::{assert_response_err, unwrap};
use fake::{Fake, Faker};
use sdk::dto::response::BodyResponseError;
use std::time::Duration;
use test_context::test_context;

use crate::helper::ApiTestContext;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_download(ctx: &mut ApiTestContext) {
  let file = ctx
    .upload_dummy_file(None, None, None, None, None, None)
    .await;
  let (status, body) = ctx.download_bytes(&file.url_path, None).await.unwrap();
  let body = unwrap!(body);
  assert_eq!(file.content, body);
  assert!(status.is_success());
}

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_download_when_file_exceed_max_dl(ctx: &mut ApiTestContext) {
  let file = ctx
    .upload_dummy_file(Some(1), None, None, None, None, None)
    .await;
  let (status, _) = ctx.download_bytes(&file.url_path, None).await.unwrap();
  assert!(status.is_success());
  let (status, resp) = ctx.download_bytes(&file.url_path, None).await.unwrap();
  assert_response_err!(resp, |e: &BodyResponseError| e.error_type == "NOT_FOUND");
  assert!(!status.is_success(), "status: {status}");
}

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_download_when_expired(ctx: &mut ApiTestContext) {
  let file = ctx
    .upload_dummy_file(None, None, Some(1), None, None, None)
    .await;
  let (status, _) = ctx.download_bytes(&file.url_path, None).await.unwrap();
  assert!(status.is_success());
  tokio::time::sleep(Duration::from_secs(1)).await;
  let (status, resp) = ctx.download_bytes(&file.url_path, None).await.unwrap();
  assert_response_err!(resp, |e: &BodyResponseError| e.error_type == "NOT_FOUND");
  assert!(!status.is_success(), "status: {status}");
}

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_download_file_with_auth(ctx: &mut ApiTestContext) {
  let auth = Some(Faker.fake::<(String, String)>());
  let file = ctx
    .upload_dummy_file(None, None, Some(1), None, None, auth.clone())
    .await;
  let (status, resp) = ctx.download_bytes(&file.url_path, None).await.unwrap();
  assert_response_err!(resp, |e: &BodyResponseError| e.error_type
    == "PERMISSION_DENIED");
  assert!(!status.is_success());
  let (status, _) = ctx.download_bytes(&file.url_path, auth).await.unwrap();
  assert!(status.is_success(), "status: {status}");
}
