use crate::{assert_response_err, assert_response_ok};
use pf_sdk::dto::{request::UploadQueryParam, response::BodyResponseError};
use test_context::test_context;

use crate::helper::ApiTestContext;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_success_upload(ctx: &mut ApiTestContext) {
  let filename = String::from("hello.txt");
  let content_type = "text/plain";
  let file = "hello".as_bytes().to_vec();
  let param: UploadQueryParam = Default::default();
  let (status, resp) = ctx
    .upload(filename, content_type, file, &param, None)
    .await
    .unwrap();
  assert_response_ok!(resp);
  assert!(status.is_success(), "status: {status}");
}

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_upload_with_invalid_len_param_query(ctx: &mut ApiTestContext) {
  let filename = String::from("hello.txt");
  let content_type = "text/plain";
  let file = "hello".as_bytes().to_vec();
  let param = UploadQueryParam {
    code_length: Some(2),
    ..Default::default()
  };
  let (status, resp) = ctx
    .upload(filename, content_type, file, &param, None)
    .await
    .unwrap();
  assert_response_err!(resp, |e: &BodyResponseError| e.error_type
    == "INVALID_INPUT");
  assert!(!status.is_success(), "status: {status}");
}
