use failure::Fail;
use std::result::Result as StdResult;

pub type Result<T = ()> = StdResult<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "parse error")]
    ParseError,
    #[fail(display = "file not exist error")]
    FileNotExistError,
    #[fail(display = "remove file error")]
    RemoveFileError,
    #[fail(display = "open file error")]
    OpenFileError,
    #[fail(display = "read to string error")]
    ReadToStringError,
    #[fail(display = "read stdin error")]
    StdinError,
    #[fail(display = "response error")]
    ResponseError,
    #[fail(display = "create file error")]
    CreateFileError,
    #[fail(display = "write to file error")]
    WriteToFileError,
}
