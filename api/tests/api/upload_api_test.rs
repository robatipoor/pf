use common::{assert_ok, model::request::UploadParamQuery};
use test_context::test_context;

use crate::helper::ApiTestContext;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_upload(ctx: &mut ApiTestContext) {
  let filename = String::from("hello.txt");
  let content_type = "text/plain";
  let file = "hello".as_bytes().to_vec();
  let query: UploadParamQuery = Default::default();
  let (status, body) = ctx
    .upload(filename, content_type, &query, file)
    .await
    .unwrap();
  assert_ok!(body);
  assert!(status.is_success(), "status: {status}");
}
