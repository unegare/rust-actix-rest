use std::{path::PathBuf, fs::DirBuilder};

use actix_multipart::{/*Field,*/ Multipart, MultipartError};
use actix_web::{
    web::{post, resource, Data, /*Json*/},
    App, FromRequest, HttpResponse, HttpServer, Result, error, /*middleware, Error*/
};

use form_data::{/*handle_multipart, *//*Field,*/ FilenameGenerator, Form, Error as FormDataError};
use futures::Future;
use futures::future::{err, Either};
use actix_http;

use rand::Rng;

use serde::{Deserialize, Serialize};

use std::io::{/*BufWriter,*/ Write/*, Read*/};

//use tokio_io::AsyncWrite;

use failure::Fail;

use image::{ImageFormat/*, guess_format*/};

use futures::stream::Stream;

//#![feature(core_intrinsics)]
//fn print_type_name<T> (t: T) {
//    println!("type_name == \"{}\"", unsafe {std::intrinsics::type_name::<T>()});
//}


const UPLOADDIR: &str = "uploaded";

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

#[derive(Debug, Fail)]
enum PIError {
    #[fail(display = "Format Guessing Error")]
    FormatGuessing(String),
//    #[fail(display = "Unsupported Format Error, {}", _0)]
    #[fail(display = "Unsupported Format Error")]
    UnsupportedFormat(String),
    #[fail(display = "Internal Server IO Error")]
    IO(String),
    #[fail(display = "Decoding Image Error")]
    Loading(String)
}

fn process_image (data: &[u8]) -> Result<String,PIError/*std::io::Error*/> {
    let fmt = match image::guess_format(&data) {
        Ok(fmt) => fmt,
        Err(e) => {
            eprintln!("process_image: {:?}", e);
            return Err(PIError::FormatGuessing(format!("{:?}", e)));
        }
    };
    let ext = match fmt {
        ImageFormat::PNG => "png",
        ImageFormat::JPEG => "jpg",
        ImageFormat::GIF => "gif",
//                                ImageFormat::WEBP => "webp",
        ImageFormat::PNM => "pnm",
//                                ImageFormat::TIFF => "tiff",
        ImageFormat::TGA => "tga",
        ImageFormat::BMP => "bmp",
        ImageFormat::ICO => "ico",
        ImageFormat::HDR => "hdr",

        _ => {
            eprintln!("process_image: match fmt: unsupported format");
            return Err(PIError::UnsupportedFormat(format!("{:?}", fmt)));
        }
    };
    let mut pb = get_random_name();
    match image::load_from_memory(&data) {
        Ok(img) => {
            let mut pbthumb = pb.clone();
            pbthumb.set_extension(String::from("thumb.") + &ext);
            match img.thumbnail_exact(100,100).save(&pbthumb) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("process_image: thumbnail_exact.save : {:?}", e);
                    return Err(PIError::IO(format!("{:?}", e)));
                }
            };
            pb.set_extension(ext);
            let mut fimg = match std::fs::File::create(&pb) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("process_image: fimg create : {:?}", e);
                    return Err(PIError::IO(format!("{:?}", e)));
                }
            };
            match fimg.write_all(&data) {
                Ok(_) => {},
                Err(e) => {
                    eprintln!("process_image: fimg.write_all : {:?}", e);
                    return Err(PIError::IO(format!("{:?}", e)));
                }
            };
        },
        Err(e) => {
            eprintln!("process_image: image::load_from_memory : {:?}", e);
            return Err(PIError::Loading(format!("{:?}", e)));
        }
    }
    Ok(pb.to_str().unwrap().to_string())
}

#[derive(Serialize)]
struct ResKeys {
    keys: Vec<String>
}

fn upload_multipart((mp, _state): (Multipart, Data<Form>)) -> Box<dyn Future<Item = HttpResponse, Error = actix_http::error::Error>> {
//    println!("state: {:?}", state);
    Box::new( 
        mp
            .map_err(error::ErrorBadRequest)
            .map(move |field: actix_multipart::Field| {
                if false {return Either::A(err(error::ErrorInternalServerError(""))).into_stream()}
                Either::B(field.fold(Vec::new(), |mut acc: Vec<u8>, b: bytes::Bytes| {
                    acc.append(&mut b.to_vec());
                    futures::lazy(move || {
                        Ok(acc)
                    })
                        .map_err(|_e: error::BlockingError<std::io::Error>| {
                            MultipartError::Incomplete
                        })
                })
                    .map(|v| {
                        actix_web::web::block(move || process_image(&v))
                            .map_err(|e: error::BlockingError<PIError>| {
                                match e {
                                    error::BlockingError::Error(e) => {
                                        match e {
                                            PIError::FormatGuessing(_) => error::ErrorBadRequest(e),
                                            PIError::UnsupportedFormat(_) => error::ErrorBadRequest(e),
                                            PIError::Loading(_) => error::ErrorBadRequest(e),
                                            PIError::IO(_) => error::ErrorInternalServerError(e),
                                        }
                                    }
                                    error::BlockingError::Canceled => error::ErrorInternalServerError("")
                                }
                            })
                    })
                    .map_err(|e: MultipartError| {
                        error::ErrorBadRequest(e)
                    })
                )
                    .into_stream()
            })
            .flatten()
            .collect()
            .and_then(|v| {
                futures::future::join_all(v)
            })
            .and_then(|v| {
//                println!("v == {:?}", v);
                HttpResponse::Created().json(ResKeys{keys: v})
            })
            .map_err(|e| {
                eprintln!("upload_multipart ended up with a error: {:?}", e);
                e
            })
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
//    println!("upload_json inside");
//    println!("{:?}", json.0);
    let mut files_data: Vec<Vec<u8>> = Vec::new();
    for fd in json.0.arr {
        match base64::decode(&fd.data) {
            Ok(data) => files_data.push(data),
            Err(e) => {
                eprintln!("upload_json: base64::decode : {:?}", e);
                return Box::new(futures::lazy(|| HttpResponse::BadRequest().finish()));
            }
        }
    }
    Box::new(
        actix_web::web::block(move || {
            let mut reskeys = ResKeys{keys: Vec::with_capacity(files_data.len())};
            for fd in files_data {
                reskeys.keys.push(process_image(&fd)?);
            }
            Ok(serde_json::to_string(&reskeys).unwrap())
        })
            .map(|s| {
                let mut res = HttpResponse::with_body(
                    http::StatusCode::CREATED,
                    actix_http::body::Body::Bytes(
                        bytes::Bytes::from(
                            s.as_bytes()
                        )
                    )
                );
                res.headers_mut().insert(http::header::CONTENT_TYPE, http::header::HeaderValue::from_str("application/json").unwrap());
                res
            })
            .map_err(|e: error::BlockingError<PIError>| {
                eprintln!("upload_json ended up with a error: {:?}", e);
                match e {
                    error::BlockingError::Error(e) => {
                        match e {
                            PIError::FormatGuessing(_) => error::ErrorBadRequest(e),
                            PIError::UnsupportedFormat(_) => error::ErrorBadRequest(e),
                            PIError::Loading(_) => error::ErrorBadRequest(e),
                            PIError::IO(_) => error::ErrorInternalServerError(e),
                        }
                    }
                    error::BlockingError::Canceled => error::ErrorInternalServerError("")
                }
            })
    )
}

fn main() -> Result<(), failure::Error> {
    let form = form_data::Form::new()
        .field("files", form_data::Field::array(
            form_data::Field::file(Gen::new())
        ));

    println!("{:?}", form);

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
                .data(form.clone())
                .route(post()
                    .to(upload_multipart)
                )
            )
            .service(resource("/upload")
                .guard(actix_web::guard::Header("content-type", "application/json"))
                .data(actix_web::web::Json::<MyJson>::configure(|cfg| {
                    cfg.limit(16777216)
                }))
                .route(post()
                    .to(upload_json)
                )
            )
    })
    .bind("0.0.0.0:8080")?
    .run()?;

    println!();

    Ok(())
}
