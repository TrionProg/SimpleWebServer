
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

    fn upload_image(self_ref:ImageApiRef, content:Vec<u8>) -> impl Future<Item = String, Error = Error> + Send + 'static {
        //TODO take format of image on upload(in

        let index = {
            let mut last_image_index_guard = self_ref.last_image_index.lock().unwrap();

            let index = *last_image_index_guard;
            *last_image_index_guard += 1;

            index
        };

        let file_name = format!("Image_{}.png", index);
        let file_path = format!("files/{}", file_name);

        let mut file = match fs::File::create(file_path.as_str()) {
            Ok(file) => file,
            Err(e) => return Either::A(err(err_msg( CanNotCreateImageFileError::from((file_path, e)) ))),
        };

        match file.write_all(content.as_ref()) {
            Ok(_) => (), //TODO
            Err(e) => return Either::A(err(err_msg(CanNotWriteImageFileError::from((file_path, e)) ))),
        }

        Either::B(ok(format!("Image has been saved as {}", file_name))) //TODO
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
        use actix_web::client::Client;

        let mut client = Client::build()
            .disable_timeout()
            .max_redirects(30)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 6.3; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/73.0.3683.103 Safari/537.36")
            .finish();

        let url_string = url.to_string();

        client.get(url)
        //client.get("http://192.168.1.132:8080/get_image/3") // <- Create request builder
            //.header("User-Agent", "Actix-web")
            //.timeout(std::time::Duration::new(180, 0))
            .send() // <- Send http request
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
        let file_name = format!("Image_{}.png", name);
        let file_path = format!("files/{}", file_name);

        let mut file = match fs::File::open(file_path.as_str()) {
            Ok(file) => file,
            Err(e) => return Either::A(err(err_msg( CanNotReadImageFileError::from((file_path, e)) ))),
        };

        //TODO BufRead

        let mut content = Vec::new();
        match file.read_to_end(&mut content) {
            Ok(_) => Either::B(ok(content)),
            Err(e) => Either::A(err(err_msg( CanNotReadImageFileError::from((file_path, e)) ))),
        }
    }

    fn get_images_list(self_ref:ImageApiRef) -> impl Future<Item = Vec<usize>, Error = Error> {
        let last_image_index = {
            let last_image_index_guard = self_ref.last_image_index.lock().unwrap();

            *last_image_index_guard
        };

        //let images_list = (0..last_image_index)

        let mut images_list = Vec::with_capacity(last_image_index);

        for i in 0..last_image_index {
            images_list.push(i);
        }

        ok(images_list)
    }
}