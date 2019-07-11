use std::path::PathBuf;

use actix_multipart::Multipart;
use actix_web::{
    web::{post, resource, Data, Json},
    App, HttpResponse, HttpServer, Result
};
use form_data::{handle_multipart, Field, FilenameGenerator, Form};
use futures::Future;
use actix_http;

use serde::{Deserialize, Serialize};

struct Gen;

impl FilenameGenerator for Gen {
    fn next_filename(&self, _: &mime::Mime) -> Option<PathBuf> {
        println!("next_filename");
        let mut p = PathBuf::new();
        p.push("examples/filename.png");
        Some(p)
    }
}

fn upload_multipart((mp, state): (Multipart, Data<Form>)) -> Box<Future<Item = HttpResponse, Error = form_data::Error>> {
    println!("state: {:?}", state);
    Box::new(
        handle_multipart(mp, state.get_ref().clone()).map(|uploaded_content| {
            println!("Uploaded Content: {:?}", uploaded_content);
            HttpResponse::Created().finish()
        }),
    )
}

#[derive(Deserialize, Serialize, Debug)]
struct FileDescriptor {
    filename: String,
    data: String
}

#[derive(Deserialize, Serialize, Debug)]
struct MyJson {
    arr: Vec<FileDescriptor>
}

fn upload_json(json: actix_web::web::Json<MyJson>) -> Box<Future<Item = HttpResponse, Error = actix_http::error::Error>> {
    println!("{:?}", json.0);
    Box::new(
        futures::lazy(|| HttpResponse::Created().finish())
    )
}


fn main() -> Result<(), failure::Error> {
    let form = Form::new()
        .field("files", Field::array(
            Field::map()
                .field("name", Field::text())
                .field("file", Field::file(Gen))
                .finalize()
        ));

    println!("{:?}", form);

    HttpServer::new(move || {
        App::new()
            .service(resource("/upload")
                .guard(actix_web::guard::fn_guard(|req| {
                    if let Some(Ok(content_type)) = req.headers().get("content-type").map(|h| h.to_str()) {
                        println!("content_type == {:?}", content_type);
                        if content_type.len() >= 19 && content_type.as_bytes()[..19].to_vec() == b"multipart/form-data" {
                            println!("True");
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }))
                .data(form.clone())
                .route(post()
                    .to(upload_multipart)
                )
            )
            .service(resource("/upload")
                .guard(actix_web::guard::Header("content-type", "application/json"))
                .data(Json(MyJson{arr: vec![]}))
                .route(post()
                    .to(upload_json)
                )
            )
    })
    .bind("127.0.0.1:8080")?
    .run()?;

    println!();

    Ok(())
}
