#[macro_use]
extern crate log;

use std::{path::PathBuf, fs::DirBuilder};

use actix_web::{
    web::{post, resource},
    App, FromRequest, HttpServer, Result 
};

use actix_rt::System;

use form_data::{/*handle_multipart, *//*Field,*/ FilenameGenerator, Form, Error as FormDataError};

use rand::Rng;

mod multipart;
use multipart::upload_multipart;

mod types;
use types::MyJson;

mod image_processing;

mod config;
use config::UPLOADDIR;

mod json;
use json::upload_json;

fn main() -> Result<(), failure::Error> {
    let form = form_data::Form::new()
        .field("files", form_data::Field::array(
            form_data::Field::file(Gen::new())
        ));

    info!("{:?}", form);

    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let sys = System::new("sys");

    HttpServer::new(move || {
        App::new()
            .service(resource("/upload")
                .guard(actix_web::guard::fn_guard(|req| {
                    if let Some(Ok(content_type)) = req.headers().get("content-type").map(|h| h.to_str()) {
                        content_type.len() >= 19 && content_type.as_bytes()[..19].to_vec() == b"multipart/form-data"
                    } else {
                        false
                    }
                }))
//                .data(ResKeys{keys: Vec::new()})
//                .data(form.clone())
                .data(Form::new())
                .route(post()
                    .to_async(upload_multipart)
                )
            )
            .service(resource("/upload")
                .guard(actix_web::guard::Header("content-type", "application/json"))
                .data(actix_web::web::Json::<MyJson>::configure(|cfg| {
                    cfg.limit(16777216)
                }))
                .route(post()
                    .to_async(upload_json)
                )
            )
    })
    .bind("0.0.0.0:8080")?
    .start();

//    actix_rt::Arbiter::spawn();

    sys.run()?;

    info!("Quitting ...");

    Ok(())
}

//macro_rules! function {
//    () => {{
//        fn f() {}
//        fn type_name_of<T>(_: T) -> &'static str {
//            std::any::type_name::<T>()
//        }
//        let name = type_name_of(f);
//        &name[..name.len() - 3]
//    }}
//}

#[inline]
fn get_random_name() -> PathBuf {
    let mut vname: [u64;4] = [0,0,0,0];
    let mut i = 0;
    while i < 4 {
        vname[i] = rand::thread_rng().gen::<u64>();
        i += 1;
    }
    let name: String = bs58::encode(safe_transmute::to_bytes::transmute_to_bytes(&vname)).into_string();
    let mut p = PathBuf::new();
    p.push(UPLOADDIR);
    p.push(name);
    p
}

#[cfg(unix)]
fn build_dir(dir: &PathBuf) -> Result<(), FormDataError> {
    use std::os::unix::fs::DirBuilderExt;

    DirBuilder::new()
        .recursive(true)
        .mode(0o755)
        .create(dir)
        .map_err(|_| FormDataError::MkDir)
}

#[cfg(not(unix))]
fn build_dir(stored_dir: &PathBuf) -> Result<(), FormDataError> {
    DirBuilder::new()
        .recursive(true)
        .create(stored_dir)
        .map_err(|_| FormDataError::MkDir)
}

struct Gen;

impl Gen {
    fn new() -> Gen {
        let mut p = PathBuf::new();
        p.push(UPLOADDIR);
        match build_dir(&p) {
            Ok(()) => {},
            Err(_e) => {
                panic!("ERROR: cannot create $UPLOADDIR");
            }
        }
        Gen
    }
}

impl FilenameGenerator for Gen {
    fn next_filename(&self, _m: &mime::Mime) -> Option<PathBuf> {
        Some(get_random_name())
    }
}

