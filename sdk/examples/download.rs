use sdk::client::PasteFileClient;

const SERVER_URL: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() {
  let client = PasteFileClient::new(SERVER_URL.to_string());
  let (status, result) = client.download("XyZ/file.txt", None).await.unwrap();
  assert!(status.is_success());
  let _ = std::str::from_utf8(&result.unwrap()).unwrap();
}
