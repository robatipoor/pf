use test_context::test_context;

use crate::helper::ApiTestContext;

#[test_context(ApiTestContext)]
#[tokio::test]
pub async fn test_download(ctx: &mut ApiTestContext) {
  let filename = String::from("hello.txt");
  let content_type = "text/plain";
  let file = "hello".as_bytes().to_vec();
  let (status, resp) = ctx
    .upload(filename, content_type, file.clone())
    .await
    .unwrap();
  assert!(status.is_success());
  let url = url::Url::parse(&resp.unwrap().url).unwrap();
  let (status, body) = ctx.download(&url.path()[1..], None).await.unwrap();
  assert!(status.is_success());
  // assert_eq!(file, body);
}
