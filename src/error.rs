use std::result;
use failure::Fail;

#[derive(Debug,Fail)]
pub enum ResponseError {
    #[fail(display="fetch file error {}",_0)]
    FetchError(String),
    #[fail(display="create file error {}",_0)]
    CreateError(String),
    #[fail(display="delete file error {}",_0)]
    DeleteError(String),
}
pub type Result<T> = result::Result<T, ResponseError>;