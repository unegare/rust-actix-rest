use futures::Future;
use actix_web::{web::Data, HttpResponse, error};
use actix_multipart::{/*Field,*/ Multipart, MultipartError};
use futures::{
    future::Either,
    stream::Stream
};
use form_data::Form;

use super::types::{PIError, ResKeys};
use super::image_processing::{process_image_fut, process_url_fut};


pub fn upload_multipart((mp, _state): (Multipart, Data<Form>)) -> impl Future<Item = HttpResponse, Error = actix_http::error::Error> {
    mp
        .map_err(error::ErrorBadRequest)
        .map(move |field: actix_multipart::Field| {
            let fieldname = match field.content_disposition() {
                Some(cd) => cd.get_name().unwrap_or("").to_owned(),
                None => "".to_string()
            };
            field.fold(Vec::new(), |mut acc: Vec<u8>, b: bytes::Bytes| {
                acc.append(&mut b.to_vec());
                futures::future::ok(acc)
                    .map_err(|_e: error::BlockingError<std::io::Error>| {
                        MultipartError::Incomplete
                    })
            })
                .map_err(|_e| PIError::IO("Multipart error".to_string()))
                .and_then(move |v: Vec<u8>| {
                    if fieldname == "url" || fieldname == "url[]" {
                        Either::A(
                            futures::lazy(|| {
                                let url = match String::from_utf8(v) {
                                    Ok(s) => s,
                                    Err(e) => {
                                        return Err(PIError::UrlParse(format!("{:?}", e)))
                                    }
                                };
                                Ok(process_url_fut(url))
                            })
                                .and_then(|fut| fut)
                        )
                    } else {
                        Either::B(process_image_fut(v))
                    }
                })
                .map_err(|e: PIError| match e {
                    PIError::BadRequest(_) | PIError::IO(_) => error::ErrorInternalServerError(e),
                    _ => error::ErrorBadRequest(e),
                })
                .into_stream()
        })
        .flatten()
        .collect()
        .and_then(|v| {
            HttpResponse::Created().json(ResKeys{keys: v})
        })
        .map_err(|e| {
            eprintln!("upload_multipart ended up with an error: {:?}", e);
            e
        })
}
