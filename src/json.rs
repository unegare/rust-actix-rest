use actix_web::{web, HttpResponse, error};
use futures::{Future, stream, stream::Stream, IntoFuture};

use super::types::{PIError, ResKeys, MyJson};
use super::image_processing::{process_image_fut, process_url_fut_binarydata};

use failure::Fail;


pub fn upload_json(json: web::Json<MyJson>) -> Box<dyn Future<Item = HttpResponse, Error = actix_http::error::Error>> {
//    println!("upload_json inside");
//    println!("{:?}", json.0);
    let mut files_data: Vec<Vec<u8>> = Vec::with_capacity(json.0.binarr.len());
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
    let urls_fut = stream::unfold(urls.into_iter(), |mut vals| {
            match vals.next() {
                Some(v) => Some(process_url_fut_binarydata(v).map(|v| (v, vals))),
                None => None,
            }
        })
            .collect()
            .into_future()
            .map(|v: Vec<Vec<u8>>| {
                stream::unfold(v.into_iter(), |mut vals| {
                    match vals.next() {
                        Some(v) => Some(process_image_fut(v).map(|v| (v, vals))),
                        None => None
                    }
                })
                    .collect()
                    .into_future()
            })
            .flatten();
//            .map(|res: Vec<String>| {
//               res 
//            })
//            .map_err(|e: PIError| e);

    let files_data_fut = stream::unfold(
            files_data.into_iter(),
            |mut vals| vals.next().map(|v| process_image_fut(v).map(|v| (v, vals)))
        )
            .collect()
            .into_future();
    Box::new(
        files_data_fut.join(urls_fut)
            .map(|(v1, v2): (Vec<String>, Vec<String>)| {
                vec![v1, v2].concat()
            })
            .map(|s: Vec<String>| {
                let mut res = HttpResponse::Created().json(ResKeys{ keys: s });
                res.headers_mut().insert(http::header::CONTENT_TYPE, http::header::HeaderValue::from_str("application/json").unwrap());
                res
            })
            .map_err(|e: PIError| match e {
                PIError::IO(s) => error::ErrorInternalServerError(s),
                y => error::ErrorBadRequest(y.name().unwrap().to_owned()),
            })
    )
}
