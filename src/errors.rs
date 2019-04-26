
#[fail(display = "Text is not valid base64")] //TODO
#[derive(Debug, Fail, Clone)]
pub struct TextNotBase64Error;

#[fail(display = "Text is not valid UTF-8")]
#[derive(Debug, Fail, Clone)]
pub struct TextNotUTF8Error;

#[fail(display = "Text is not valid URL")]
#[derive(Debug, Fail, Clone)]
pub struct TextNotURLError;

#[fail(display = "Can not create image file \"{}\" error: \"{}\"", path, error)]
#[derive(Debug, Fail, Clone)]
pub struct CanNotCreateImageFileError{
    pub path:String,
    pub error:String
}

impl From<(String, std::io::Error)> for CanNotCreateImageFileError {
    fn from((path, error):(String, std::io::Error)) -> Self {
        CanNotCreateImageFileError {
            path,
            error:format!("{}",error)
        }
    }
}

#[fail(display = "Can not write image file \"{}\" error: \"{}\"", path, error)]
#[derive(Debug, Fail, Clone)]
pub struct CanNotWriteImageFileError{
    pub path:String,
    pub error:String
}

impl From<(String, std::io::Error)> for CanNotWriteImageFileError {
    fn from((path, error):(String, std::io::Error)) -> Self {
        CanNotWriteImageFileError {
            path,
            error:format!("{}",error)
        }
    }
}

#[fail(display = "Can not read image file \"{}\" error: \"{}\"", path, error)]
#[derive(Debug, Fail, Clone)]
pub struct CanNotReadImageFileError{
    pub path:String,
    pub error:String
}

impl From<(String, std::io::Error)> for CanNotReadImageFileError {
    fn from((path, error):(String, std::io::Error)) -> Self {
        CanNotReadImageFileError {
            path,
            error:format!("{}",error)
        }
    }
}

#[fail(display = "Unsupported image format, supported:Png, Jpeg")]
#[derive(Debug, Fail, Clone)]
pub struct UnsupportedImageFormatError;

#[fail(display = "Can not parse \"{}\" as number", string)]
#[derive(Debug, Fail, Clone)]
pub struct CanNotParseAsNumberError{
    pub string:String
}

impl From<&str> for CanNotParseAsNumberError {
    fn from(string:&str) -> Self {
        CanNotParseAsNumberError {
            string:string.to_string()
        }
    }
}

#[fail(display = "Can not download by given URL \"{}\" error: \"{}\"", url, error)]
#[derive(Debug, Fail, Clone)]
pub struct CanNodDownloadByURLError{
    pub url:String,
    pub error:String
}

use actix_web::http::StatusCode;

impl From<(String, StatusCode)> for CanNodDownloadByURLError {
    fn from((url, error):(String, StatusCode)) -> Self {
        CanNodDownloadByURLError {
            url,
            error:format!("{}",error)
        }
    }
}

impl From<(String, String)> for CanNodDownloadByURLError {
    fn from((url, error):(String, String)) -> Self {
        CanNodDownloadByURLError {
            url,
            error
        }
    }
}