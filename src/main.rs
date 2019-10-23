extern crate actix_web;
extern crate futures;

//#[macro_use]
extern crate failure;

#[macro_use]
extern crate failure_derive;

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

pub enum ToMainMessage{
    CtrlC
}

fn main() {
    use std::sync::mpsc;
    use std::thread;

    use openssl_probe::*;
    init_ssl_cert_env_vars();

    let dirs = find_certs_dirs();

    for dir in dirs.iter() {
        println!("OpenSSL_ dir {:?}", dir);
    }

    let mut result = probe();

    match result.cert_dir {
        Some(ref dir) => println!("OpenSSL dir {:?}", dir),
        None => {}
    }

    match result.cert_file {
        Some(ref dir) => println!("OpenSSL file {:?}", dir),
        None => {}
    }

    /*
    match probe() {
        Ok(result) => {
            match result.cert_dir {
                Some(ref dir) => println!("OpenSSL dir {}", dir),
                None => {}
            }
        },
        Err(_) => {}
    }
    */



    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    let image_api_ref = image_api::ImageApi::new_ref();

    let (main_tx, main_rx) = mpsc::channel();
    let (http_server_tx, http_server_rx) = mpsc::channel();

    let thread_join_handle = thread::spawn(move || {
        let sys = actix_rt::System::new("http-server");

        let http_server = HttpServer::new(move|| {
            let image_app = ImageApp::new(image_api_ref.clone());

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
        });

        let http_server = match http_server.bind("127.0.0.1:8080") {
            Ok(http_server) => http_server,
            Err(e) => {
                http_server_tx.send(Err(format!("Can not start HTTP server: {}", e))).unwrap();
                return;
            }
        };

        let http_server_handle = http_server
            .shutdown_timeout(5)
            .disable_signals()
            .start();

        http_server_tx.send(Ok((http_server_handle, actix_rt::System::current()))).unwrap();

        let _ = sys.run();
    });

    let (server_addr, system) = match http_server_rx.recv().unwrap() {
        Ok((http_server_handle, system)) => (http_server_handle, system),
        Err(e) => {
            println!("{}",e);
            return;
        }
    };

    ctrlc::set_handler(move || {
        match main_tx.send(ToMainMessage::CtrlC) {
            Ok(_) => {},
            Err(e) => {}
        }
    });

    loop {
        match main_rx.recv(){
            Ok(message) => {
                match message {
                    ToMainMessage::CtrlC => break,
                }
            },
            Err(_) => break
        }
    }

    println!("Stopping server");
    server_addr.stop(true);
    system.stop();

    thread_join_handle.join();
    println!("bye bye");
}