pub mod file;

use axum::extract::BodyStream;
use axum_extra::body::AsyncReadBody;
use common::error::ApiResult;
use futures_util::TryStreamExt;
use tokio::fs::File;
use tokio::io::BufWriter;
use tokio_util::io::StreamReader;

use crate::server::ApiState;

pub async fn store_stream(
  state: &ApiState,
  file_name: &str,
  stream: BodyStream,
) -> ApiResult<String> {
  let stream = stream.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));
  let stream = StreamReader::new(stream);
  futures_util::pin_mut!(stream);
  let mut file = BufWriter::new(File::create(file_name).await?);
  tokio::io::copy(&mut stream, &mut file).await?;
  Ok("".to_string())
}

pub async fn get(code: &str, file_name: &str) -> ApiResult<AsyncReadBody<File>> {
  // let file = File::open(path).await?;
  // Ok(AsyncReadBody::new(file))
  todo!()
}
