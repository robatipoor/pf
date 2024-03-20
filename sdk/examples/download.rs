use pf_sdk::{client::PasteFileClient, dto::FileUrlPath};

const SERVER_URL: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() {
  let client = PasteFileClient::new(SERVER_URL.to_string());
  let url_path = FileUrlPath {
    code: "example-code".to_string(),
    file_name: "example_file_name.txt".to_string(),
  };
  let (status, result) = client.download_bytes(&url_path, None).await.unwrap();
  assert!(status.is_success());
  let _ = std::str::from_utf8(&result.unwrap()).unwrap();
}
