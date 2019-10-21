
use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::{error, middleware, web, App, HttpResponse, HttpServer, Responder};
use actix_web::Error as ActixError;

use futures::future::{err, ok, Either, IntoFuture};
use futures::{Future, Stream};

use failure::Error;

use actix_web::HttpRequest;

use image_api::ImageApi;
use image_api::ImageApiRef;
use image_api::PutImageInput;

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
    ) -> Box<Future<Item = HttpResponse, Error = ActixError>> {
        let branch = multipart
            .map_err(error::ErrorInternalServerError)
            .map(|field| Self::read_field(field).into_stream())
            .flatten()
            .filter(|field| field.is_some())
            .map(|field| field.unwrap())
            .collect()
            .and_then(move|fields| {
                use futures::stream::iter_ok;

                let app_copy = app.clone();
                let app_copy2 = app.clone();

                iter_ok::<_, ()>(fields)
                    .map(move |field|{
                        app_copy.process_put_image(field).then(|answer_result|{
                            match answer_result {
                                Ok(answer) => Ok(Ok(answer)),
                                Err(error) => Ok(Err(error))
                            }
                        })
                        .into_stream()
                    })
                    .flatten()
                    .collect()
                    .and_then(move |answers_result|{
                        let mut errors = Vec::new();
                        let mut answers = Vec::new();

                        for answer_result in answers_result {
                            match answer_result {
                                Ok(Some(answer)) => answers.push(answer),
                                Ok(None) => {},
                                Err(e) => errors.push(e)
                            }
                        }

                        //TODO где же asnwers, errors?
                        app_copy2.create_page(answers, errors).then(|page_result|{
                            match page_result {
                                Ok(page) => HttpResponse::Ok().content_type("text/html").body(page),
                                Err(error) => HttpResponse::Ok().content_type("text/html").body(format!("Error: {}",error))
                            }
                        })
                    })
            })
            .map_err(|e| {
                println!("failed: {}", e);
                e
            });

        Box::new(branch)
    }

    fn read_field(field: Field) -> impl Future<Item = Option<(String, Vec<u8>)>, Error = ActixError> {
        match field.content_disposition() {
            Some(disposition) => {
                let field_name =match disposition.get_name() {
                    Some(name) => name.to_string(),
                    None => return Either::A(err(error::ErrorBadGateway("aaa1"))),//TODO
                };

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

    fn process_put_image(&self, (name, value):(String, Vec<u8>)) -> impl Future<Item = Option<String>, Error = Error> {
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
            if value.len() > 0 { //TODO read_field if buffer.len() > 0 {
                let fut = ImageApi::put_image(api, PutImageInput::Content(value)).map(|answer| Some(answer));
                Either::A(Either::A(Either::B(fut)))
            }else{
                Either::A(Either::B(ok(None)))
            }
        }else{
            Either::A(Either::B(ok(None)))
        }
    }

    fn create_page(&self, answers:Vec<String>, errors:Vec<failure::Error>) -> impl Future<Item = String, Error = Error> {
        let api = self.api.clone();

        ok(String::new()).and_then(move |mut page|{
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

            Ok(page)
        }).and_then(move |mut page| {
            ImageApi::get_images_list(api).then(move |image_list_result|{
                match image_list_result {
                    Ok(image_list) => Ok((page, image_list)),
                    Err(error) => {
                        page.push_str("<div class=\"error\">");

                        let error_string = format!("{}", error);
                        page.push_str(error_string.as_str());

                        page.push_str("</div><br>");

                        Ok((page, Vec::new()))
                    }
                }
            })
        }).and_then(|(mut page, image_list)|{
            for image_index in image_list.iter().rev() {
                page.push_str("<div class=\"image_box\">");

                let image_name = format!("Image_{}", image_index);
                page.push_str(image_name.as_str());

                let image_tag = format!("<img width=\"400px\" src=\"get_image/{0}\" alt=\"Image {0}\">",image_index);
                page.push_str(image_tag.as_str());
                let image_tag = format!("<img width=\"400px\" src=\"get_image/{0}mini\" alt=\"Image {0}mini\">",image_index);
                page.push_str(image_tag.as_str());

                page.push_str("</div>");
            }

            page.push_str(include_str!("../html/page_end.html"));

            Ok(page)
        })
    }

    pub fn show_image(app: web::Data<ImageApp>) -> Box<Future<Item = HttpResponse, Error = ActixError>> {
        let branch = app.create_page(Vec::new(), Vec::new()).then(|page_result|{
            match page_result {
                Ok(page) => HttpResponse::Ok().content_type("text/html").body(page),
                Err(error) => HttpResponse::Ok().content_type("text/html").body(format!("Error: {}",error))
            }
        });

        Box::new(branch)
    }

    pub fn get_image(req: HttpRequest, app: web::Data<ImageApp>) -> Box<Future<Item = HttpResponse, Error = ActixError>> {
        let name = req.match_info().query("name");

        if name.len() > 0 {
            let api = app.api.clone();

            let branch = ImageApi::get_image(api, name).then(move |result|{
                match result {
                    Ok(content) => HttpResponse::Ok().content_type("image/png").body(content),
                    Err(e) => HttpResponse::Ok().body("error")//TODO text of error
                }
            });

            Box::new(branch)
        }else{
            let branch = HttpResponse::Ok().body("empty request");//TODO text of error

            Box::new(ok(branch))
        }
    }
}