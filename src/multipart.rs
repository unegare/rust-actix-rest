use futures::Future;
use actix_web::{web::Data, client::Client, HttpResponse, error};
use actix_multipart::{/*Field,*/ Multipart, MultipartError};
use futures::{
    future::{err, Either},
    stream::Stream
};
use form_data::Form;

use super::types::{PIError, ResKeys};
use super::image_processing::process_image;


pub fn upload_multipart((mp, _state): (Multipart, Data<Form>)) -> impl Future<Item = HttpResponse, Error = actix_http::error::Error> {
//    println!("state: {:?}", state);
//    Box::new( 
        mp
            .map_err(error::ErrorBadRequest)
            .map(move |field: actix_multipart::Field| {
//                println!("cd : {:?}", field.content_disposition());
                let fieldname = match field.content_disposition() {
                    Some(cd) => match cd.get_name() {
                        Some(name) => String::from(name),
                        None => "".to_string()
                    },
                    None => "".to_string()
                };
//                println!("fieldname == {}", fieldname);
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
//                        multipart_process_url(url)
                        actix_web::web::block(move || {
                                if fieldname == "url" || fieldname == "url[]" {
//                                    println!("url : {:?}", v);
                                    let url = match String::from_utf8(v) {
                                        Ok(s) => s,
                                        Err(e) => {
//                                            eprintln!("String::from_utf8::Err: {:?}", e);
                                            return Err(PIError::UrlParse(format!("{:?}", e)))
                                        }
                                    };
//                                    println!("url: {}", url);
                                    let res = std::thread::spawn(move || { //wrapped in a thread because of a bug in actix .... actix issue #1007
                                        actix_rt::System::new("fut").block_on(futures::lazy(|| {
                                            let client = Client::new();
                                            client.get(url)
                                                .send()
                                                .map_err(|e| {
                                                    eprintln!("client err, {:?}", e);
                                                    error::PayloadError::Overflow //just a error
                                                })
                                                .and_then(|mut res| {
//                                                    println!("Response: {:?}", res);
                                                    res.body()
                                                })
                                                .map_err(|e| PIError::BadRequest(e.to_string()))
                                                .and_then(|body| {
                                                    process_image(&body.to_vec())
                                                })
                                            })
                                        )
                                    }).join();
                                    match res {
                                        Ok(s) => {return s;},
                                        Err(e) => {
                                            eprintln!("err: {:?}", e);
                                            return Err(PIError::IO("".to_string()));
                                        }
                                    };
                                } else {
                                    process_image(&v)
                                }
                            })
                            .map_err(|e: error::BlockingError<PIError>| {
                                match e {
                                    error::BlockingError::Error(e) => {
                                        match e {
                                            PIError::FormatGuessing(_) => error::ErrorBadRequest(e),
                                            PIError::UnsupportedFormat(_) => error::ErrorBadRequest(e),
                                            PIError::Loading(_) => error::ErrorBadRequest(e),
                                            PIError::IO(_) => error::ErrorInternalServerError(e),
                                            PIError::UrlParse(_) => error::ErrorBadRequest(e),
//                                            PIError::BadContentType(_) => error::ErrorBadRequest(e),
                                            PIError::BadRequest(_) => error::ErrorInternalServerError(e)
                                        }
                                    }
                                    error::BlockingError::Canceled => error::ErrorInternalServerError("Canceled")
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
//    )
}
