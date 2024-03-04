use sdk::{client::PasteFileClient, dto::request::UploadQueryParam};

const SERVER_URL: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() {
  let client = PasteFileClient::new(SERVER_URL.to_string());
  let file = "Hello World!".as_bytes().to_vec();
  let param = UploadQueryParam {
    ..Default::default()
  };
  let (status, _result) = client
    .upload("file".to_string(), "text", file, &param, None)
    .await
    .unwrap();
  assert!(status.is_success())
}
