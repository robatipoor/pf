use sdk::{error::BodyResponseError, model::request::UploadParamQuery};
use test_context::test_context;
use util::{assert_err, assert_ok};

use crate::helper::ApiTestContext;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_success_upload(ctx: &mut ApiTestContext) {
  let filename = String::from("hello.txt");
  let content_type = "text/plain";
  let file = "hello".as_bytes().to_vec();
  let query: UploadParamQuery = Default::default();
  let (status, resp) = ctx
    .upload(filename, content_type, &query, file, None)
    .await
    .unwrap();
  assert_ok!(resp);
  assert!(status.is_success(), "status: {status}");
}

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_upload_with_invalid_len_param_query(ctx: &mut ApiTestContext) {
  let filename = String::from("hello.txt");
  let content_type = "text/plain";
  let file = "hello".as_bytes().to_vec();
  let query = UploadParamQuery {
    code_length: Some(2),
    ..Default::default()
  };
  let (status, resp) = ctx
    .upload(filename, content_type, &query, file, None)
    .await
    .unwrap();
  assert_err!(resp, |e: &BodyResponseError| e.error_type
    == "INVALID_INPUT");
  assert!(!status.is_success(), "status: {status}");
}
