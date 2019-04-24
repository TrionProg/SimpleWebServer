
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
            //...map(Self::read_form)
            .flatten()
            .filter(|field| field.is_some())
            .map(|field| field.unwrap())
            .collect()
            .and_then(move|fields| {
                //тут мы должны подготовить данные и вызывать апи, и уже после вызова мы можем венуть response
                //тут две три стадии: извлечь инфу, вызвать апи, вернуть респонз
                /*
                let mut text = None;
                let mut file = None;

                let mut errors = Vec::new();
                */

                /*
                for field in fields.into_iter() {
                    if field.name.as_ref() == "text" {//TODO to static/const
                        let string = String::from_utf8(field.value).unwrap();//TODO

                        println!("{}", string);
                    }
                }
                */

                /*
                for (name, value) in fields {
                    if name.as_str() == "text" {//TODO to static/const
                        match String::from_utf8(value) {
                            Ok(value) => text = Some(value),
                            Err(_) => errors.push("text is not valid utf-8"),
                        }
                    }else if name.as_str() == "file" {
                        file = Some(value);
                    }
                }

                HttpResponse::Ok().body("haha")
                */

                /*
                app.process_put_image(fields).and_then(|body|{
                    HttpResponse::Ok().body(body.as_str())
                }).map_err(|e|{
                    HttpResponse::Ok().body("error")
                })
                */

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

        /*
        multipart
            .map_err(error::ErrorInternalServerError)
            //.map(Self::read_form)
            .map(|i|{
                //let a = Self::read_form(i);
                //a

                let r = if false {
                    Err(error::ErrorInternalServerError)
                }else{
                    Ok(1i64)
                };

                //let r : Result<i64, Error> = Ok(1);

               // ok(1)
                Box::new(r.into_future())
            })
            //.flatten()
            .collect()
            .map(|sizes:i64| HttpResponse::Ok().body("aaaa"))
            .map_err(|e| {
                println!("failed: {}", e);
                e
            })
        */
    }

    fn read_field(field: Field) -> impl Future<Item = Option<(String, Vec<u8>)>, Error = Error> {
        /*
        ok(field).and_then(|field|{
            match field.content_disposition() {
                Some(disposition) => Ok((field, disposition)),
                None => Err(err(error::ErrorBadGateway("aaa")))
            }
        }).and_then(|(field, disposition)|{
            match disposition.get_name() {
                Some(name) => Ok((field, name.to_string())),
                None => Err(err(error::ErrorBadGateway("aaa")))
            }
        });
        */

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

    /*
    fn read_form(item: Item) -> Box<Stream<Item = Option<(String, Vec<u8>)>, Error = Error>> {
        match item {
            Item::Field(field) => Box::new(Self::read_field(field).into_stream()),
            Item::Nested(mp) => Box::new(
                mp.map_err(error::ErrorInternalServerError)
                    .map(Self::read_form)
                    .flatten(),
            ),
        }
    }
    */

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
                    /*
                    Either::B(Ok(Either::A(ok(()).and_then(|_|{
                        Ok(format!("{} hhha", "blaa"))
                    })))
*/
                    //Ok(format!("{} hhha", "blaa"))

                    Either::B(Either::A(ImageApi::put_image(api, PutImageInput::Content(content)).then(|result|{
                        match result {
                            Ok(response) => Ok(response),
                            Err(e) => Err(Error::from(e))
                        }
                    })))

                    /*
                    Either::B(
                        ok(file).and_then(|file|{
                            self.api.put_image()
                        })
                    )
                    */
                },
                None => {
                    match text {
                        Some(text) => {
                            /*
                            Ok(Either::B(ok(texto).and_then(|texti|{
                                Ok(format!("{} hhha", texti))
                            })))
                            */

                            //Ok(format!("{} hhha", "blaa"))
                            //Either::B(ok(format!("{} hhha", "blaa")))

                            Either::B(Either::B(ImageApi::put_image(api, PutImageInput::Text(text)).then(|result|{
                                match result {
                                    Ok(response) => Ok(response),
                                    Err(e) => Err(Error::from(e))
                                }
                            })))
                            /*
                            Either::B(ImageApi::put_image(api).then(|result|{
                                Ok(response) => Ok(response),
                                Err(e) => Error::from(e)
                            }))
                            */
                        }
                        None => {
                            Either::A(err(error::ErrorBadGateway("bbb")))//TODO
                        }
                    }
                }
            }
        })

            /*
        ok(Either::A(ok(()).and_then(|_|{
            Ok(format!("{} hhha", "blaa"))
        })))
        */

        /*
        match file {
            Some(ref file) => {
                //ok("haha".to_string())
                ok(Either::A(ok(()).and_then(|_|{
                    Ok(format!("{} hhha", "blaa"))
                })))
            },
            None => {
                match text {
                    Some(texto) => {
                        ok(Either::B(ok(texto).and_then(|texti|{
                            Ok(format!("{} hhha", texti))
                        })))
                    }
                    None => {
                        err(error::ErrorBadGateway("aaa"))
                    }
                }
            }
        }
        */
    }

    /*
    fn read_form(item: Item) -> Box<Stream<Item = i64, Error = Error>> {
        match item {
            Item::Field(field) => Box::new(Self::read_field(field).into_stream()),
            Item::Nested(mp) => Box::new(
                mp.map_err(error::ErrorInternalServerError)
                    .map(Self::read_form)
                    .flatten(),
            ),
        }
    }

    fn read_field(field: Field) -> impl Future<Item = i64, Error = Error> {
        match field.content_disposition() {
            Some(disposition) => {
                match disposition.get_name() {
                    Some(name) => println!("{}", name),
                    None => {println!("2")}
                }
            },
            None => {println!("1")}
        }


        let file_path_string = "upload.png";
        let mut file = match fs::File::create(file_path_string) {
            Ok(file) => file,
            Err(e) => return Either::A(err(error::ErrorInternalServerError(e)))
        };

        Either::B(
            field
                .fold(0i64, move |acc, bytes| {
                    file.write_all(bytes.as_ref())
                        .map(|_| acc + bytes.len() as i64)
                        .map_err(|e| {
                            println!("file.write_all failed: {:?}", e);
                            MultipartError::Payload(error::PayloadError::Io(e))
                        })
                })
                .map_err(|e| {
                    println!("read_field failed, {:?}", e);
                    error::ErrorInternalServerError(e)
                }),
        )
    }
    */

        /*
        pub fn put_image(app: web::Data<ImageApp>,) -> impl Responder {
            println!("sadsa");
            //let answer = String::extract(req);


            "aasdas".to_string()
        }
        */

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

    /*
    fn process_get_image(&self, name:&str) -> impl Future<Item = Vec<u8>, Error = Error> {
        let api = self.api.clone();

            /*
        ok(()).and_then(

        )

        Either::B(Either::A(ImageApi::put_image(api, PutImageInput::Content(content)).then(|result|{
            match result {
                Ok(response) => Ok(response),
                Err(e) => Err(Error::from(e))
            }
        })))
            */

        ImageApi::get_image(api, name).then(move |result|{
            match result {
                Ok(content) => Ok(content),//TODO with format of image
                Err(e) => Err(Error::from(e))
            }
        })

    }
    */

    /*
    pub fn get_image(app: web::Data<ImageApp>) -> impl Responder {
        let name = Path::<String>::extract();
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
    */
}