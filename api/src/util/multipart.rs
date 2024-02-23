use axum::{
  body::{Body, Bytes},
  extract::{FromRequest, Multipart, Request},
};
use hyper::header::CONTENT_TYPE;

pub async fn create_multipart_request(file_name: &str, data: &str) -> anyhow::Result<Multipart> {
  let data = format!("--X-BOUNDARY\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\n\r\n{data}\r\n--X-BOUNDARY--\r\n");
  let body = Body::from(Bytes::from(data));
  let request = Request::builder()
    .header(CONTENT_TYPE, "multipart/form-data; boundary=X-BOUNDARY")
    .body(body)?;
  Ok(Multipart::from_request(request, &()).await?)
}
