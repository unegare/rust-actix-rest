use serde::{Deserialize, Serialize};
use failure::Fail;

#[derive(Debug, Fail)]
pub enum PIError {
    #[fail(display = "Format Guessing Error")]
    FormatGuessing(String),
//    #[fail(display = "Unsupported Format Error, {}", _0)]
    #[fail(display = "Unsupported Format Error")]
    UnsupportedFormat(String),
    #[fail(display = "Internal Server IO Error")]
    IO(String),
    #[fail(display = "Decoding Image Error")]
    Loading(String),
    #[fail(display = "Url Parse Error")]
    UrlParse(String),
//    #[fail(display = "Unsuitable Content Type Error")]
//    BadContentType(String),
    #[fail(display = "Bad Request ERROR {}", 0)]
    BadRequest(String)
}

#[derive(Serialize)]
pub struct ResKeys {
    pub keys: Vec<String>
}

#[derive(Deserialize, Debug)]
pub struct MyJson {
    pub binarr: Vec<FileDescriptor>,
    pub urls: Vec<String>
}

#[derive(Deserialize, Debug)]
pub struct FileDescriptor {
    pub filename: String,
    pub data: String
}
