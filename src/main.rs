extern crate actix_web;
extern crate futures;
extern crate failure;

pub mod image_api;
use image_api::ImageApi;

pub mod image_app;
use image_app::ImageApp;


//use std::cell::Cell;
use std::fs::{self};
use std::io::Write;

use actix_multipart::{Field, Item, Multipart, MultipartError};
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer};
use futures::future::{err, Either};
use futures::{Future, Stream};

/*
pub struct AppState {
    pub counter: Cell<usize>,
}
*/

pub fn save_file(field: Field) -> impl Future<Item = i64, Error = Error> {
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
                println!("save_file failed, {:?}", e);
                error::ErrorInternalServerError(e)
            }),
    )
}

pub fn handle_multipart_item(item: Item) -> Box<Stream<Item = i64, Error = Error>> {
    match item {
        Item::Field(field) => Box::new(save_file(field).into_stream()),
        Item::Nested(mp) => Box::new(
            mp.map_err(error::ErrorInternalServerError)
                .map(handle_multipart_item)
                .flatten(),
        ),
    }
}

pub fn upload(
    multipart: Multipart,
    counter: web::Data<ImageApp>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    //counter.set(counter.get() + 1);
    //println!("{:?}", counter.get());

    multipart
        .map_err(error::ErrorInternalServerError)
        .map(handle_multipart_item)
        .flatten()
        .collect()
        .map(|sizes| HttpResponse::Ok().json(sizes))
        .map_err(|e| {
            println!("failed: {}", e);
            e
        })
}

fn index() -> HttpResponse {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form target="/" method="post" enctype="multipart/form-data">
                <input type="file" name="file"/>
                <input type="submit" value="Submit"></button>
            </form>
        </body>
    </html>"#;

    HttpResponse::Ok().body(html)
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    let image_api_ref = ImageApi::new_ref();

    HttpServer::new(move|| {
        let image_app = ImageApp::new(image_api_ref.clone());

        /*
        App::with_state(image_app)
            .resource("/", |r|
                r.method(http::Method::GET).f(ImageApp::show_image)
            ).resource("/put_image", |r|
            r.method(http::Method::POST).f(ImageApp::put_image)
            );
        */

        App::new()
            .data(image_app)
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/")
                    .route(web::get().to(ImageApp::show_image))
                    .route(web::post().to_async(ImageApp::put_image)),
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
}

/*extern crate actix_web;
extern crate futures;
extern crate failure;

pub mod image_api;
use image_api::ImageApi;

pub mod image_app;
use image_app::ImageApp;


use actix_web::{server, App, HttpRequest, Responder};

/*
fn greet(req: &HttpRequest) -> impl Responder {

    //How to insert Api?
    let to = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", to)
}
*/

use actix_web::{http};
use std::cell::Cell;

/*
// This struct represents state
struct ImageApp {
    counter: Cell<usize>,
}

fn index(req: &HttpRequest<AppState>) -> String {
    let count = req.state().counter.get() + 1; // <- get count
    req.state().counter.set(count); // <- store new count in state

    format!("Request number: {}", count) // <- response with count
}
*/

fn main() {
    let image_api_ref = ImageApi::new_ref();

    server::new(move|| {
        //let image_api_ref = ImageApi::new_ref();
        let image_app = ImageApp::new(image_api_ref.clone());

        App::with_state(image_app)
            .resource("/", |r|
                r.method(http::Method::GET).f(ImageApp::show_image)
            ).resource("/put_image", |r|
                r.method(http::Method::POST).f(ImageApp::put_image)
            )
    }).bind("127.0.0.1:8080")
        .unwrap()
        .run();

    /*
    server::new(|| {
        //let api_ref_ref = &api_ref;
        App::new()
            .resource("/", |r| r.f(greet))
            .resource("/{name}", |r| r.f(greet))
    })
        */
    /*
    App::with_state(ImageApp::new(image_api_ref))
        .resource("/", |r| r.method(http::Method::GET).f(ImageApp::put_image))
        .finish()
        .bind("127.0.0.1:8080")
        .unwrap()
        .run();
    */

    /*
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run();*/
}
*/