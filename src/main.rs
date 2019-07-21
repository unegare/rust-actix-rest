use std::path::PathBuf;

use actix_multipart::{Field, Multipart, MultipartError};
use actix_web::{
    web::{post, resource, Data, Json},
    App, FromRequest, HttpResponse, HttpServer, Result, error, middleware, Error
};

use form_data::{handle_multipart, /*Field,*/ FilenameGenerator, Form};
use futures::Future;
use futures::future::{err, Either};
use actix_http;

use rand::Rng;

use serde::{Deserialize, Serialize};

use std::io::{BufWriter, Write, Read};

//use tokio_io::AsyncWrite;

use image::{ImageFormat, guess_format};

//mod my_handle_multipart;

use futures::stream::Stream;

struct Gen;

//std::arc::Arc<u64> counter;
//std::sync::atomic::AtomicU64

//#![feature(core_intrinsics)]
//fn print_type_name<T> (t: T) {
//    println!("type_name == \"{}\"", unsafe {std::intrinsics::type_name::<T>()});
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

fn upload_multipart((mp, state): (Multipart, Data<Form>)) -> Box<dyn Future<Item = HttpResponse, Error = actix_http::error::Error>> { //Error = form_data::Error>> {
    println!("state: {:?}", state);
    Box::new( 
        mp
            .map_err(error::ErrorInternalServerError)
            .map(move |field| {
                if 1 == 0 {return Either::A(err(error::ErrorInternalServerError(""))).into_stream()}
                Either::B(field.fold(("".to_string(), Vec::new()), |(_noname, mut acc): (String, Vec<u8>), b: bytes::Bytes| {
                    println!("after fold ... bytes.len() == {} | da.len() == {}", b.len(), acc.len());
                    acc.append(&mut b.to_vec());
                    /*actix_web::web::block*/futures::lazy(move || {
                        Ok(("ok".to_string(), acc))
                    })
                        .map_err(|e: error::BlockingError<MultipartError>| {
                            eprintln!("map_err: {:?}", e);
                            MultipartError::Incomplete
                        })
                })
                    .map(|(_s, v)| {
                        println!("v.len() == {}", v.len());
                        actix_web::web::block(move || {
                            let fmt = match image::guess_format(&v) {
                                Ok(fmt) => fmt,
                                Err(e) => {
                                    eprintln!("{:?}", e);
                                    return Err(MultipartError::Incomplete);
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
                                    eprintln!("match fmt: unsupported format");
                                    return Err(MultipartError::Incomplete);                                }
                            };
                            let mut pb = get_random_name();
                            match image::load_from_memory(&v) {
                                Ok(img) => {
                                    let mut pbthumb = pb.clone();
                                    pbthumb.set_extension(String::from("thumb.") + &ext);
                                    match img.thumbnail(100,100).save(&pbthumb) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            eprintln!("upload_multipart: thumbnail.save : {:?}", e);
                                            return Err(MultipartError::Incomplete);
                                        }
                                    };
                                    pb.set_extension(ext);
                                    match img.save(&pb) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            eprintln!("upload_multipart: img.save : {:?}", e);
                                            return Err(MultipartError::Incomplete);
                                        }
                                    }
                                },
                                Err(e) => {
                                    eprintln!("upload_multipart: image::load_from_memory : {:?}", e);
                                    return Err(MultipartError::Incomplete);
                                }
                            }
                            Ok(pb.to_str().unwrap().to_string())
                        })
                            .map(|s| s)
                            .map_err(|e: error::BlockingError<MultipartError>| {MultipartError::Incomplete})
                    })
                    .map_err(|e| error::ErrorInternalServerError(e))
                )
                    .into_stream()
            })
            .flatten()
            .collect()
            .map_err(|e| {
                eprintln!("failed1: {:?}", e);
                MultipartError::Incomplete
            })
            .and_then(|v| futures::future::join_all(v))
            .map_err(|e| {
                eprintln!("failed2: {:?}", e);
                error::ErrorInternalServerError(e)
            })
            .and_then(|v| {
                println!("v == {:?}", v);
                HttpResponse::Created().json(ResKeys{keys: v})
            })
            .map_err(|e| {
                println!("failed3: {:?}", e);
                error::ErrorInternalServerError(e)
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
    println!("upload_json inside");
//    println!("{:?}", json.0);
    Box::new(
        futures::lazy(|| {
            let mut files_data: Vec<Vec<u8>> = Vec::new();
            for fd in json.0.arr {
                match base64::decode(&fd.data) {
                    Ok(data) => files_data.push(data),
                    Err(e) => {
                        println!("Bad 1");
                        println!("{:?}", e);
                        return HttpResponse::BadRequest().finish();
                    }
                }
            }
            let mut reskeys = ResKeys{keys: Vec::with_capacity(files_data.len())};
//            let mut vfut = Vec::new();
            for fd in files_data {
                let fmt = match guess_format(&fd) {
                    Ok(fmt) => fmt,
                    _ => {
                        println!("Bad 2");
                        return HttpResponse::with_body(http::StatusCode::BAD_REQUEST, actix_http::body::Body::Bytes(bytes::Bytes::from("unsupported img format")));
                    }
                };
                let mut pb = get_random_name();
                match fmt {
                    ImageFormat::PNG => {
                        pb.set_extension("png")
                    },
                    ImageFormat::JPEG => pb.set_extension("jpg"),
                    ImageFormat::GIF => pb.set_extension("gif"),
//                    ImageFormat::WEBP => pb.set_extension(".webp"),
                    ImageFormat::PNM => pb.set_extension("pnm"),
//                    ImageFormat::TIFF => pb.set_extension(".tiff"),
                    ImageFormat::TGA => pb.set_extension("tga"),
                    ImageFormat::BMP => pb.set_extension("bmp"),
                    ImageFormat::ICO => pb.set_extension("ico"),
                    ImageFormat::HDR => pb.set_extension("hdr"),

                    _ => {
                        println!("Bad 3");
                        return HttpResponse::with_body(http::StatusCode::BAD_REQUEST, actix_http::body::Body::Bytes(bytes::Bytes::from("unsupported img format")));
                    }
                };
                let mut f = match std::fs::File::create(&pb) {
                    Ok(handler) => BufWriter::new(handler),
                    _ => {
                        eprintln!("couldn't create the file {:?}", &pb);
                        return HttpResponse::InternalServerError().finish();
                    }
                };

                match image::load_from_memory(&fd) {
                    Ok(img) => {
                        let mut pb_thumb = pb.clone();
                        pb_thumb.set_extension("thumb.png");
                        match img.thumbnail(100,100).save(pb_thumb) {
                            Ok(_) => {},
                            Err(e) => {
                                eprintln!("thumbnail save error: {:?}", e);
                                return HttpResponse::InternalServerError().finish();
                            }
                        };
                    },
                    Err(e) => {
                        eprintln!("error: {:?}", e);
                        return HttpResponse::InternalServerError().finish();
                    }
                };

//                if let Ok(handler) = std::fs::File::create(&pb) {
//                    f = BufWriter::new(handler);
//                } else {
//                    eprintln!("couldn't create the file {:?}", &pb);
//                    return HttpResponse::InternalServerError().finish();
//                }
//                vfut.push(f.write_buf(&fd));
//                vfut.push(AsyncWrite::write_buf(&mut f, &fd));
                if let Ok(_) = f.write(&fd) {
                } else {
                    eprintln!("write error into the file {:?}", &pb);
                    return HttpResponse::InternalServerError().finish();
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
    let form = form_data::Form::new()
        .field("files", form_data::Field::array(
            form_data::Field::file(Gen)
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
//                .data(ResKeys{keys: Vec::new()})
                .data(form.clone())
//                .data(Form::new())
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
    .bind("127.0.0.1:8080")?
    .run()?;

    println!();

    Ok(())
}
