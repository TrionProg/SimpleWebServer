
use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer, Responder};
use futures::future::{err, ok, Either, IntoFuture};
use futures::{Future, Stream};

use actix_web::HttpRequest;
/*
use failure::Error;

use actix_web::{web, App, HttpRequest, HttpResponse, Responder};
use actix_multipart::{Field, Item, Multipart, MultipartError};
use futures::future::{err, Either};
use futures::{Future, Stream};
use actix_web::{error};
*/

use crate::image_api::ImageApi;
use crate::image_api::ImageApiRef;
use crate::image_api::PutImageInput;

/*
use std::fs;
use std::io::Write;
use core::borrow::Borrow;
*/

#[derive(Clone)]
pub struct ImageApp {
    api:ImageApiRef
}

impl ImageApp {
    pub fn new(api:ImageApiRef) -> Self {
        ImageApp {
            api
        }
    }

    pub fn put_image(
        multipart: Multipart,
        app: web::Data<ImageApp>,
    ) -> Box<Future<Item = HttpResponse, Error = Error>> {//impl Future<Item = HttpResponse, Error = Error> {
        let branch = multipart
            .map_err(error::ErrorInternalServerError)
            .map(|field| Self::read_field(field).into_stream())
            .flatten()
            .filter(|field| field.is_some())
            .map(|field| field.unwrap())
            .collect()
            .and_then(move|fields| {
                app.process_put_image(fields).then(move |result|{
                    match result {
                        Ok(response) => HttpResponse::Ok().body(response),
                        Err(e) => HttpResponse::Ok().body(format!("{}", e))
                    }
                })
            })
            .map_err(|e| {
                println!("failed: {}", e);
                e
            });

        Box::new(branch)
    }

    fn read_field(field: Field) -> impl Future<Item = Option<(String, Vec<u8>)>, Error = Error> {
        match field.content_disposition() {
            Some(disposition) => {
                let field_name =match disposition.get_name() {
                    Some(name) => name.to_string(),
                    None => return Either::A(err(error::ErrorBadGateway("aaa1"))),//TODO
                };

                println!("{}", field_name);

                let a = field.fold(Vec::new(), move |mut buffer, bytes| {
                    buffer.extend_from_slice(bytes.as_ref());

                    let r:Result<Vec<u8>, actix_multipart::MultipartError> = Ok(buffer);

                    r
                }).map_err(|e| {
                    println!("read_field failed, {:?}", e);
                    error::ErrorInternalServerError(e)
                }).and_then(|buffer|{
                    if buffer.len() > 0 {
                        Ok(Some((field_name, buffer)))
                    }else{
                        Ok(None)
                    }
                });

                Either::B(Either::A(a))
            },
            None => Either::B(Either::B(ok(None)))
        }
    }

    fn process_put_image(&self, fields:Vec<(String, Vec<u8>)>) -> impl Future<Item = String, Error = Error> {
        let api = self.api.clone();

        ok(fields).and_then(|fields|{
            let mut text = None;
            let mut content = None;

            for (name, value) in fields {
                if name.as_str() == "text" {//TODO to static/const
                    match String::from_utf8(value) {
                        Ok(value) => {
                            if value.len() > 0 {
                                text = Some(value)
                            }
                        },
                        Err(_) => return Err(error::ErrorBadGateway("aaa==")),//TODO
                    }
                }else if name.as_str() == "file" {
                    if value.len() > 0 {
                        content = Some(value);
                    }
                }
            }

            Ok((text, content))
        }).and_then(|(text, content)|{
            match content {
                Some(content) => {
                    Either::B(Either::A(ImageApi::put_image(api, PutImageInput::Content(content)).then(|result|{
                        match result {
                            Ok(response) => Ok(response),
                            Err(e) => Err(Error::from(e))
                        }
                    })))
                },
                None => {
                    match text {
                        Some(text) => {
                            Either::B(Either::B(ImageApi::put_image(api, PutImageInput::Text(text)).then(|result|{
                                match result {
                                    Ok(response) => Ok(response),
                                    Err(e) => Err(Error::from(e))
                                }
                            })))
                        }
                        None => {
                            Either::A(err(error::ErrorBadGateway("bbb")))//TODO
                        }
                    }
                }
            }
        })
    }

    //TODO to upload_image_form
    pub fn show_image(app: web::Data<ImageApp>) -> impl Responder {
        let body = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form target="/" method="post" enctype="multipart/form-data">
            <input type="text" name="text" size="40">
            <p><input type="radio" name="answer" value="a1">Офицерский состав<Br>
  <input type="radio" name="answer" value="a2">Операционная система<Br>
                <input type="file" name="file"/>
                <input type="submit" value="Submit"></button>
            </form>
        </body>
        </html>"#;

        HttpResponse::Ok().content_type("text/html").body(body)
    }

    pub fn get_image(req: HttpRequest, app: web::Data<ImageApp>) -> Box<Future<Item = HttpResponse, Error = Error>> {
        let name = req.match_info().query("name");

        if name.len() > 0 {
            let api = app.api.clone();

            let branch = ImageApi::get_image(api, name).then(move |result|{
                match result {
                    Ok(content) => HttpResponse::Ok().content_type("image/png").body(content),
                    Err(e) => HttpResponse::Ok().body("error")
                }
            });

            Box::new(branch)
        }else{
            let branch = HttpResponse::Ok().body("empty request");//TODO

            Box::new(ok(branch))
        }
    }
}