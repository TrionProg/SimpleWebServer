extern crate actix_web;
extern crate futures;

//#[macro_use]
extern crate failure;

#[macro_use]
extern crate failure_derive;

use libc::{c_void, c_char, size_t, c_int};

#[repr(C)]
pub struct cv_return_value_void_X {
    pub error_code: i32,
    pub error_msg: *const c_char,
    pub result: *mut c_void
}


//#[link(name = "opencv_world346", kind = "static")]
#[link(name = "target/debug/opencv_world340")]
extern "C"{
    pub fn cvLoadImage(filename: *const c_char, flags: i32) -> *mut c_void;
    //pub fn cv_imgcodecs_cv_imwrite_String_filename_Mat_img_VectorOfint_params(filename: *const c_char, flags: i32) -> Mat;//*mut c_void;
    //pub fn imread(filename: *const c_char, flags: i32) -> Mat;//*mut c_void;
    pub fn cvSaveImage(filename: *const c_char, arr: *const c_void, param: *const c_int) -> c_int;
    //pub fn cv_imgcodecs_cv_imread_String_filename_int_flags(filename: *const c_char, flags: i32) -> cv_return_value_void_X;
    //fn snappy_max_compressed_length(source_length: size_t) -> size_t;
    pub fn cvResize(src: *const c_void, dst: *const c_void, interpolation:c_int) -> c_void;
    pub fn cvCreateImage(size:Size, depth:c_int, channels:c_int) -> *mut c_void;
}

#[allow(dead_code)]
pub struct Mat {
    #[doc(hidden)] pub ptr: *mut c_void
}

#[repr(C)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

use std::ffi::{CString, CStr};

fn convert() {
    unsafe {
        let file_name = "files/Image_1.png";

        let rv = cvLoadImage(CString::new(file_name).unwrap().as_ptr(), 1);

        println!("-1");
        let mini = cvCreateImage ( Size{width:200, height:100}, 8, 3 );

        println!("-2");
        cvResize(rv, mini, 1);

        println!("aaa {}", rv as usize);

        let p:[c_int;3] = [
            1, 10, 0
        ];

        /*
        c_int p[3];
        IplImage * img = cvLoadImage("test.jpg");

        p[0] = CV_IMWRITE_JPEG_QUALITY;
        p[1] = 10;
        p[2] = 0;
        */

        //let res = cvSaveImage(CString::new("hello.png").unwrap().as_ptr(), rv, p.as_ptr());
        let res = cvSaveImage(CString::new("files/mini.png").unwrap().as_ptr(), mini, 0 as *const c_int);

        println!("{}", res);

    }
}

/*
pub fn imread2(filename:&str, flags: i32) -> Result<Mat,String> {
    unsafe {
        println!("--");
        let rv = cvLoadImage(CString::new(filename).unwrap().as_ptr(), flags);

        println!("aaa {}", rv as usize);

        Ok(Mat{ ptr: rv })

        /*
        if rv.error_msg as i32 != 0i32 {
            let v = CStr::from_ptr(rv.error_msg).to_bytes().to_vec();
            ::libc::free(rv.error_msg as *mut c_void);
            return Err(String::from_utf8(v).unwrap())
        }

        Ok(Mat{ ptr: rv.result })
        */
    }
}
*/


pub mod image_api;
use image_api::ImageApi;

pub mod image_app;
use image_app::ImageApp;

pub mod errors;


use std::cell::Cell;
use std::fs::{self};
use std::io::Write;

use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::{error, middleware, web, App, Error, HttpResponse, HttpServer};
use futures::future::{err, Either};
use futures::{Future, Stream};

/*

pub struct AppState {
    pub counter: Cell<usize>,
}

pub fn save_file(field: Field) -> impl Future<Item = i64, Error = Error> {
    let file_path_string = "upload.png";
    let mut file = match fs::File::create(file_path_string) {
        Ok(file) => file,
        Err(e) => return Either::A(err(error::ErrorInternalServerError(e))),
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

pub fn upload(
    multipart: Multipart,
    counter: web::Data<Cell<usize>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    counter.set(counter.get() + 1);
    println!("{:?}", counter.get());

    multipart
        .map_err(error::ErrorInternalServerError)
        .map(|field| save_file(field).into_stream())
        .flatten()
        .collect()
        .map(|sizes| HttpResponse::Ok().json(sizes))
        .map_err(|e| {
            println!("failed: {}", e);
            e
        })
}

*/

fn main() -> std::io::Result<()> {

    /*
    use std::mem::transmute;

    unsafe {
        let a = std::mem::transmute::<fn cvLoadImage(filename: *const c_char, flags: i32) -> cv_return_value_void_X, usize > (cvLoadImage);
    }
    */
        /*
    use actix_rt::System;
    use actix_web::client::Client;

    System::new("test").block_on(lazy(|| {
        let mut client = Client::default();

        client.get("http://www.rust-lang.org") // <- Create request builder
            .header("User-Agent", "Actix-web")
            .send()                             // <- Send http request
            .map_err(|_| ())
            .and_then(|response| {              // <- server http response
                println!("Response: {:?}", response);
                Ok(())
            })
    }));

    Ok(())
    */

    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();


    convert();
    /*
    match imread2("files/Image_1.png", 1) {
        Ok(_) => {},
        Err(e) => println!("error {}", e)
    }
    */

    println!("Contin");

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
            ).service(
            web::resource("/get_image/{name}")
                .route(web::get().to(ImageApp::get_image))
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