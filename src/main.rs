use std::{path::PathBuf, fs::DirBuilder};

use actix_web::{
    web::{post, resource},
    App, FromRequest, HttpServer, Result 
};

use form_data::{Error as FormDataError};

mod multipart;
use multipart::upload_multipart;

mod types;
use types::MyJson;

mod image_processing;

mod config;
use config::UPLOADDIR;

mod json;
use json::upload_json;

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

fn main() -> Result<(), failure::Error> {
    build_dir(&PathBuf::from(UPLOADDIR))?;

    println!("To realize how to use it take a look at \"./example/client.sh\"");

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
    .run()?;

    println!("Exiting ..."); //gracefull shutdown is within the actix framework.

    Ok(())
}
