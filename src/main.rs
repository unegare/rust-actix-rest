use std::path::PathBuf;

use actix_multipart::Multipart;
use actix_web::{
    web::{post, resource, Data, Json},
    App, HttpResponse, HttpServer, Result
};
use form_data::{handle_multipart, Field, FilenameGenerator, Form};
use futures::Future;
use actix_http;

use rand::Rng;

use serde::{Deserialize, Serialize};

use std::io::{BufWriter, Write};

//use tokio_io::AsyncWrite;

struct Gen;

//std::arc::Arc<u64> counter;
//std::sync::atomic::AtomicU64

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
    p.push(format!("uploaded/{}", name));
    p
}


impl FilenameGenerator for Gen {
    fn next_filename(&self, _m: &mime::Mime) -> Option<PathBuf> {
//        println!("next_filename");
//        println!("m: {:?}", m);
        Some(get_random_name())
    }
}

#[derive(Serialize)]
struct ResKeys {
    keys: Vec<String>
}

fn upload_multipart((mp, state): (Multipart, Data<Form>)) -> Box<dyn Future<Item = HttpResponse, Error = form_data::Error>> {
//    println!("state: {:?}", state);
    Box::new(
        handle_multipart(mp, state.get_ref().clone()).map(|uploaded_content| {
            println!("Uploaded Content: {:?}", uploaded_content);
            let mut sres = ResKeys{keys: Vec::new()};
            if let form_data::Value::Map(h) = uploaded_content {
                if let Some(form_data::Value::Array(arr)) = h.get("files") {
                    for arr_item in arr {
                        if let form_data::Value::File(_src, dst) = arr_item {
//                            println!("{:?}, {:?}", src, dst);
                            sres.keys.push(dst.to_str().unwrap().to_string());
                        } else {
                        }
                    }
                } else {
                }
            } else {
            }
            let mut res = HttpResponse::with_body(
                http::StatusCode::CREATED,
                actix_http::body::Body::Bytes(
                    bytes::Bytes::from(
                        serde_json::to_string(&sres).unwrap().as_bytes()
                    )
                )
            );
//            res.headers_mut().inner.insert(http::header::HeaderName::from(b"Content-Type"), actix_http::header::Value::One(http::header::HeaderValue));
            res.headers_mut().insert(http::header::CONTENT_TYPE, http::header::HeaderValue::from_str("application/json").unwrap());
            res
        }),
    )
}

#[derive(Deserialize, Debug)]
struct FileDescriptor {
    filename: String,
    data: String
}

#[derive(Deserialize, Debug)]
struct MyJson {
    arr: Vec<FileDescriptor>
}

fn upload_json(json: actix_web::web::Json<MyJson>) -> Box<dyn Future<Item = HttpResponse, Error = actix_http::error::Error>> {
    println!("{:?}", json.0);
    Box::new(
        futures::lazy(|| {
            let mut files_data: Vec<Vec<u8>> = Vec::new();
            for fd in json.0.arr {
                if let Ok(data) = base64::decode(&fd.data) {
                    files_data.push(data);
                } else {
                    return HttpResponse::BadRequest().finish();
                }
            }
            let mut reskeys = ResKeys{keys: Vec::with_capacity(files_data.len())};
//            let mut vfut = Vec::new();
            for fd in files_data {
                let pb = get_random_name();
                let mut f;
                if let Ok(handler) = std::fs::File::create(&pb) {
                    f = BufWriter::new(handler);
                } else {
                    eprintln!("couldn't create the file {:?}", &pb);
                    return HttpResponse::InternalServerError().finish();
                }
//                vfut.push(f.write_buf(&fd));
//                vfut.push(AsyncWrite::write_buf(&mut f, &fd));
                if let Ok(_) = f.write(&fd) {
                } else {
                    eprintln!("write error into the file {:?}", &pb);
                }
                reskeys.keys.push(pb.to_str().unwrap().to_string());
            }
//            futures::future::join_all(vfut).wait();
            let mut res = HttpResponse::with_body(
                http::StatusCode::CREATED,
                actix_http::body::Body::Bytes(
                    bytes::Bytes::from(
                        serde_json::to_string(&reskeys).unwrap().as_bytes()
                    )
                )
            );
            res.headers_mut().insert(http::header::CONTENT_TYPE, http::header::HeaderValue::from_str("application/json").unwrap());
            res
        })
    )
}

fn main() -> Result<(), failure::Error> {
    let form = Form::new()
        .field("files", Field::array(
            Field::file(Gen)
//            Field::map()
//                .field("key", Field::text())
//                .field("file", Field::file(Gen))
//                .finalize()
        ));

    println!("{:?}", form);

    HttpServer::new(move || {
        App::new()
            .service(resource("/upload")
                .guard(actix_web::guard::fn_guard(|req| {
                    if let Some(Ok(content_type)) = req.headers().get("content-type").map(|h| h.to_str()) {
                        println!("content_type == {:?}", content_type);
                        content_type.len() >= 19 && content_type.as_bytes()[..19].to_vec() == b"multipart/form-data"
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
                .data(Json(MyJson{arr: Vec::new()}))
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
