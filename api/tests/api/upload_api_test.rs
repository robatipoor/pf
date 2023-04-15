use test_context::test_context;

use crate::helper::ApiTestContext;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_upload(ctx: &mut ApiTestContext) {
  let filename = String::from("hello.txt");
  let content_type = "text/plain";
  let file = "hello".as_bytes().to_vec();
  let (status, body) = ctx.upload(filename, content_type, file).await.unwrap();
  assert!(body.is_ok());
  assert!(status.is_success());
}
