use actix_web::{web, client::Client, HttpResponse, error};
use futures::Future;

use super::types::{PIError, ResKeys, MyJson};
use super::image_processing::process_image;


pub fn upload_json(json: web::Json<MyJson>) -> Box<dyn Future<Item = HttpResponse, Error = actix_http::error::Error>> {
//    println!("upload_json inside");
//    println!("{:?}", json.0);
    let mut files_data: Vec<Vec<u8>> = Vec::new();
    for fd in json.0.binarr {
        match base64::decode(&fd.data) {
            Ok(data) => files_data.push(data),
            Err(e) => {
                eprintln!("upload_json: base64::decode : {:?}", e);
                return Box::new(futures::lazy(|| HttpResponse::BadRequest().finish()));
            }
        }
    }
    let urls: Vec<String> = json.0.urls;
    Box::new(
        actix_web::web::block(move || {
            let mut reskeys = ResKeys{keys: Vec::with_capacity(files_data.len())};
            for fd in files_data {
                reskeys.keys.push(process_image(&fd)?);
            }
            for link in urls {
                reskeys.keys.push(process_url(link)?);
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
                            PIError::UrlParse(_) => error::ErrorBadRequest(e),
//                            PIError::BadContentType(_) => error::ErrorBadRequest(e),
                            PIError::BadRequest(_) => error::ErrorInternalServerError(e)
                        }
                    }
                    error::BlockingError::Canceled => error::ErrorInternalServerError("")
                }
            })
    )
}

fn process_url (url: String) -> Result<String, PIError>{
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
//                    println!("Response: {:?}", res);
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
        Ok(s) => s,
        Err(e) => {
            eprintln!("err: {:?}", e);
            Err(PIError::IO("".to_string()))
        }
    }
}
