use crate::helper::ApiTestContext;
use crate::{assert_response_err, unwrap};
use fake::{Fake, Faker};
use sdk::dto::{response::BodyResponseError, FileUrlPath};
use test_context::test_context;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_info(ctx: &mut ApiTestContext) {
  let file = ctx
    .upload_dummy_file(None, None, None, None, None, None)
    .await;
  let (status, resp) = ctx.info(&file.url_path, None).await.unwrap();
  let resp = unwrap!(resp);
  assert!(status.is_success(), "status: {status}");
  assert!(resp.allow_manual_deletion);
}

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_get_info_when_file_not_exist(ctx: &mut ApiTestContext) {
  let url_path = Faker.fake::<FileUrlPath>();
  let (status, resp) = ctx.info(&url_path, None).await.unwrap();
  assert_response_err!(resp, |e: &BodyResponseError| e.error_type == "NOT_FOUND");
  assert!(!status.is_success(), "status: {status}");
}

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_get_info_when_file_exceed_max_dl(ctx: &mut ApiTestContext) {
  let file = ctx
    .upload_dummy_file(Some(1), None, None, None, None, None)
    .await;
  let (status, _resp) = ctx.download(&file.url_path, None).await.unwrap();
  assert!(status.is_success());
  let (status, resp) = ctx.info(&file.url_path, None).await.unwrap();
  assert_response_err!(resp, |e: &BodyResponseError| e.error_type == "NOT_FOUND");
  assert!(!status.is_success(), "status: {status}");
}
