
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
                use futures::stream::iter_ok;

                iter_ok::<_, ()>(fields)
                    .map(move |field|{
                        app.process_put_image(field).then(|answer_result|{
                            match answer_result {
                                Ok(answer) => Ok(Ok(answer)),
                                Err(error) => Ok(Err(error))
                            }
                        })
                        .into_stream()
                    })
                    .flatten()
                    .collect()
                    .and_then(|answers_result|{
                        let mut errors = Vec::new();
                        let mut answers = Vec::new();

                        for answer_result in answers_result {
                            match answer_result {
                                Ok(Some(answer)) => answers.push(answer),
                                Ok(None) => {},
                                Err(e) => errors.push(e)
                            }
                        }

                        let page = Self::create_page(answers, errors);

                        HttpResponse::Ok().content_type("text/html").body(page)
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

    fn process_put_image(&self, (name, value):(String, Vec<u8>)) -> impl Future<Item = Option<String>, Error = failure::Error> {
        let api = self.api.clone();

        use failure::err_msg;

        if name.as_str() == "text" {//TODO to static/const
            match String::from_utf8(value) {
                Ok(value) => {
                    if value.len() > 0 {
                        let fut = ImageApi::put_image(api, PutImageInput::Text(value)).map(|answer| Some(answer));

                        Either::A(Either::A(Either::A(fut)))
                    }else{
                        Either::A(Either::B(ok(None)))
                    }
                },
                Err(_) => return Either::B(err(err_msg("aaa==")))//TODO
            }
        }else if name.as_str() == "file" {
            if value.len() > 0 {
                let fut = ImageApi::put_image(api, PutImageInput::Content(value)).map(|answer| Some(answer));
                Either::A(Either::A(Either::B(fut)))
            }else{
                Either::A(Either::B(ok(None)))
            }
        }else{
            Either::A(Either::B(ok(None)))
        }
    }

    fn create_page(answers:Vec<String>, errors:Vec<failure::Error>) -> String {
        let mut page = String::new();

        page.push_str(include_str!("../html/page_begin.html"));

        if answers.len() > 0 {
            page.push_str("<div class=\"success\">");

            for answer in answers.iter() {
                page.push_str(answer.as_str());
                page.push_str("<br>");
            }

            page.push_str("</div><br>");
        }

        if errors.len() > 0 {
            page.push_str("<div class=\"error\">");

            for error in errors.iter() {
                let error_string = format!("{}", error);
                page.push_str(error_string.as_str());
                page.push_str("<br>");
            }

            page.push_str("</div><br>");
        }

        page.push_str(include_str!("../html/form.html"));

        page.push_str(include_str!("../html/page_end.html"));

        page
    }

    //TODO to upload_image_form
    pub fn show_image(app: web::Data<ImageApp>) -> impl Responder {
        let page = Self::create_page(Vec::new(), Vec::new());

        HttpResponse::Ok().content_type("text/html").body(page)
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