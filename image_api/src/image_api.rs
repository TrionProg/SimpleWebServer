
use futures::{future, Future, Stream, Poll, Async};
use futures::future::{err, ok, Either};
use failure::Error;
use failure::err_msg;
//use actix_web::Error;

use crate::errors::*;

use std::sync::Arc;
use std::sync::Mutex;
//use actix_net::service::ServiceExt;

use actix_web::client::{ClientResponse, ClientRequest};

use std::fs;
use std::io::Write;
use std::io::Read;
use core::borrow::Borrow;

use url::Url;

pub type ImageApiRef = Arc<ImageApi>;

pub struct ImageApi {
    last_image_index: Mutex<usize>
}

pub enum PutImageInput {
    Text(String),
    Content(Vec<u8>)
}

pub enum ImageFormat {
    Png,
    Jpeg
}

impl ImageApi {
    pub fn new() -> Self {
        ImageApi {
            last_image_index: Mutex::new(1)
        }
    }

    pub fn new_ref() -> ImageApiRef {
        Arc::new(Self::new())
    }

    pub fn put_image(self_ref:ImageApiRef, image:PutImageInput) -> Box<Future<Item = String, Error = Error>> {
        let branch = match image {
            PutImageInput::Content(content) =>
                Either::A(Self::upload_image(self_ref, content)),
            PutImageInput::Text(text) => {
                match Self::is_image_base64(text.as_str()) {
                    Some((format, content_base64)) =>
                        Either::B(Either::A(Self::upload_image_base64(self_ref, content_base64))),
                    None => {
                        match Url::parse(text.as_str()) {
                            Ok(_) =>
                                Either::B(Either::B(Either::A(Self::download_image(self_ref, text.as_str())))),
                            Err(_) =>
                                Either::B(Either::B(Either::B(err(err_msg(TextNotBase64Error)))))
                        }
                    }
                }
            }
        };

        Box::new(branch)
    }

    //TODO Невстроенная
    fn upload_image(self_ref:ImageApiRef, content:Vec<u8>) -> impl Future<Item = String, Error = Error> + Send + 'static {
        //TODO take format of image on upload

        let index = {
            let mut last_image_index_guard = self_ref.last_image_index.lock().unwrap();

            let index = *last_image_index_guard;
            *last_image_index_guard += 1;

            index
        };

        ok(content).and_then(move |content|{
            let file_name = format!("Image_{}.png", index);
            let file_path = format!("files/{}", file_name);

            match Self::write_image(file_path.as_str(), &content) {
                Ok(_) => Ok((index, file_name, file_path)),
                Err(e) => Err(e)
            }
        }).and_then(|(index, file_name, file_path)|{
            let file_mini_name = format!("Image_{}mini.png", index);
            let file_mini_path = format!("files/{}", file_mini_name);

            match Self::resize_image(file_path.as_str(), file_mini_path.as_str()) {
                Ok(_) => Ok((file_name, file_mini_name)),
                Err(e) => Err(e)
            }
        }).and_then(|(file_name, file_mini_name)| {
            let message = format!("Image has been saved as {} and {}", file_name, file_mini_name);

            ok(message)
        })
    }

    fn write_image(file_path:&str, content:&Vec<u8>) -> Result<(), Error> {
        use std::io::BufWriter;

        let mut file = match fs::File::create(file_path) {
            Ok(file) => file,
            Err(e) => return Err(err_msg( CanNotCreateImageFileError::from((file_path.to_string(), e)) )),
        };

        let mut writer = BufWriter::new(file);

        match writer.write_all(content.as_ref()) {
            Ok(_) => (), //TODO
            Err(e) => return Err(err_msg(CanNotWriteImageFileError::from((file_path.to_string(), e)) )),
        }

        Ok(())
    }

    fn resize_image(file_path:&str, file_mini_path:&str) -> Result<(), Error> {
        use opencv::image::Image;
        use opencv::image::Size;

        let rv = Image::open(file_path)?;

        let mini = Image::create(Size{width:200, height:200}, 8, 3)?;

        let mini = rv.resize(mini)?;

        mini.save(file_mini_path)?;

        Ok(())
    }

    fn is_image_base64(text:&str) -> Option<(ImageFormat, &str)> {
        if text.starts_with("data:image/png;base64,") {
            let (a, b) = text.split_at("data:image/png;base64,".len());

            Some((ImageFormat::Png, b))
        }else if text.starts_with("data:image/jpeg;base64,") {
            let (a, b) = text.split_at("data:image/jpeg;base64,".len());

            Some((ImageFormat::Jpeg, b))
        }else{
            None
        }
    }

    fn upload_image_base64(self_ref:ImageApiRef, content_base64:&str) -> impl Future<Item = String, Error = Error> {
        use base64::decode;

        match decode(content_base64) {
            Ok(content) => Either::A(Self::upload_image(self_ref, content)),
            Err(e) => Either::B(err(err_msg(TextNotBase64Error)))
        }
    }

    fn download_image(self_ref:ImageApiRef, url:&str) -> impl Future<Item = String, Error = Error> {
        let url_string = url.to_string();

        use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
        use awc::Connector as SuperConnector;
        use awc::ClientBuilder;
        use std::time::Duration;

        let mut ssl_conn_builder = SslConnector::builder(SslMethod::tls()).unwrap();
        //Не проверять сертификат сервера, почему-то эта проверка как правило фейлится
        ssl_conn_builder.set_verify(SslVerifyMode::NONE);
        let ssl_connector = ssl_conn_builder.build();

        let connector = SuperConnector::new()
            .ssl(ssl_connector)
            .timeout(Duration::from_secs(50))
            .finish();
        let ssl_client = ClientBuilder::new()
            .timeout(Duration::from_secs(50))
            .connector(connector)
            .finish();

        ssl_client.get(url)
            //.header("Content-Type", "text/html")
            .header("User-Agent", "Actix-web")
            .send()
            .then(|result|{
                match result {
                    Ok(mut response) => {
                        if response.status().is_success() {
                            let mut format = None;
                            let mut content_length = 0;

                            for (key, val) in response.headers().iter() {
                                if key == "content-type" {
                                    if val == "image/png" {
                                        format = Some(ImageFormat::Png);
                                    }else if val == "image/jpeg" {
                                        format = Some(ImageFormat::Jpeg);
                                    }else{
                                        return Either::B(err(err_msg(UnsupportedImageFormatError)));
                                    }
                                }else if key == "content-length" {
                                    match std::str::from_utf8(val.as_ref()) {
                                        Ok(number_string) => {
                                            match number_string.parse::<usize>() {
                                                Ok(len) => content_length=1000000,
                                                Err(_) => return Either::B(err(err_msg( CanNotParseAsNumberError::from((number_string)) )))
                                            }
                                        },
                                        Err(_) => return Either::B(err(err_msg(TextNotUTF8Error)))
                                    }
                                }
                            }

                            match format {
                                Some(format) => {
                                    let fut = response.body().limit(content_length).then(|result|{
                                        match result {
                                            Ok(bytes) => {
                                                let mut content = Vec::new();
                                                content.extend_from_slice(bytes.as_ref());

                                                Either::A(Self::upload_image(self_ref, content))
                                            },
                                            Err(e) => Either::B(err(err_msg("can not read body")))
                                        }
                                    });

                                    Either::A(fut)
                                },
                                None => Either::B(err(err_msg(UnsupportedImageFormatError)))
                            }
                        }else{
                            Either::B(err(err_msg( CanNodDownloadByURLError::from((url_string, response.status())) )))
                        }
                    },
                    Err(e) =>
                        Either::B(err(err_msg( CanNodDownloadByURLError::from((url_string, format!("{}",e))) )))
                }
            })
    }

    pub fn get_image(self_ref:ImageApiRef, name:&str) -> Box<Future<Item = Vec<u8>, Error = Error>> {
        Box::new(Self::load_image(self_ref, name))
    }

    fn load_image(self_ref:ImageApiRef, name:&str) -> impl Future<Item = Vec<u8>, Error = Error> {
        use std::io::BufReader;

        let file_name = format!("Image_{}.png", name);
        let file_path = format!("files/{}", file_name);

        let mut file = match fs::File::open(file_path.as_str()) {
            Ok(file) => file,
            Err(e) => return Either::A(err(err_msg( CanNotReadImageFileError::from((file_path, e)) ))),
        };

        let mut reader = BufReader::new(file);

        let mut content = Vec::new();
        match reader.read_to_end(&mut content) {
            Ok(_) => Either::B(ok(content)),
            Err(e) => Either::A(err(err_msg( CanNotReadImageFileError::from((file_path, e)) ))),
        }
    }

    //TODO невстроенная
    pub fn get_images_list(self_ref:ImageApiRef) -> impl Future<Item = Vec<usize>, Error = Error> {
        let last_image_index = {
            let last_image_index_guard = self_ref.last_image_index.lock().unwrap();

            *last_image_index_guard
        };

        let mut images_list = Vec::with_capacity(last_image_index);

        for i in 1..last_image_index {
            images_list.push(i);
        }

        ok(images_list)
    }
}