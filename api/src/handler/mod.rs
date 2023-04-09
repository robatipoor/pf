pub mod file;

use axum::{
  body::{boxed, StreamBody},
  extract::{BodyStream, Path, State},
  response::{Html, Response},
  Json,
};
use common::{error::ApiResult, model::response::MessageResponse};

use crate::{server::ApiState, service};

pub async fn health_check() -> Json<MessageResponse> {
  Json(MessageResponse {
    message: "OK".to_string(),
  })
}

pub async fn home_page() -> Html<&'static str> {
  Html(
    r#"
        <!doctype html>
        <html>
            <head>
                <title>Upload something!</title>
            </head>
            <body>
                <form action="/" method="post" enctype="multipart/form-data">
                    <div>
                        <label>
                            Upload file:
                            <input type="file" name="file" multiple>
                        </label>
                    </div>
                    <div>
                        <input type="submit" value="Upload files">
                    </div>
                </form>
            </body>
        </html>
        "#,
  )
}
