
use futures::{future, Future, Stream, Poll, Async};
use futures::future::{err, ok, Either};
use failure::Error;
use failure::err_msg;
//use actix_web::Error;

use std::sync::Arc;
use std::sync::Mutex;
//use actix_net::service::ServiceExt;

use std::fs;
use std::io::Write;
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

impl ImageApi {
    pub fn new() -> Self {
        ImageApi {
            last_image_index: Mutex::new(1)
        }
    }

    pub fn new_ref() -> ImageApiRef {
        Arc::new(Self::new())
    }

    pub fn put_image(self_ref:ImageApiRef, image:PutImageInput) -> Box<Future<Item = String, Error = Error> + Send + 'static> {
        let branch = match image {
            PutImageInput::Content(content) =>
                Either::A(Self::upload_image(self_ref, content)),
            PutImageInput::Text(text) => {
                match Url::parse(text.as_str()) {
                    Ok(_) => {
                        Either::B(Either::A(ok("is url".to_string())))
                    },
                    Err(_) => {
                        Either::B(Either::B(ok("not url".to_string())))
                    }
                }
            }
        };

        Box::new(branch)
        /*
        let fut = future::ok(self_ref).and_then(|self_ref|{
            println!("aaa");

            future::ok("aaa".to_string())
        });

        Box::new(fut)
        */
    }

    fn upload_image(self_ref:ImageApiRef, content:Vec<u8>) -> impl Future<Item = String, Error = Error> + Send + 'static {
        let index = {
            let mut last_image_index_guard = self_ref.last_image_index.lock().unwrap();

            let index = *last_image_index_guard;
            *last_image_index_guard += 1;

            index
        };

        let file_name = format!("Image_{}.png", index);

        let mut file = match fs::File::create(file_name.as_str()) {
            Ok(file) => file,
            Err(e) => return Either::A(err(err_msg("can not save image"))), //TODO
        };

        match file.write_all(content.as_ref()) {
            Ok(_) => (), //TODO
            Err(e) => return Either::A(err(err_msg("can not save image"))), //TODO
        }

        Either::B(ok(format!("Image has been saved as {} <br>", file_name))) //TODO

    }
}