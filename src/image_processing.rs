use actix_web::{web, error::BlockingError, client::Client, error};
use std::{path::PathBuf, io::Write};
use image::{ImageFormat/*, guess_format*/};
use rand::Rng;

use futures::Future;

use super::types::PIError;

use super::config::UPLOADDIR;

pub fn process_image_fut (data: Vec<u8>) -> impl Future<Item = String, Error = PIError> {
    web::block(move || {
        process_image(&data)
    })
        .map_err(|e: BlockingError<PIError>| match e {
            BlockingError::Error(e) => e,
            BlockingError::Canceled => PIError::IO("BlockingError::Cancelled".to_string()),
        })
}

pub fn process_image (data: &[u8]) -> Result<String,PIError/*std::io::Error*/> {
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

#[allow(dead_code)]
pub fn process_url_fut (url: String) -> impl Future<Item = String, Error = PIError> {
    let client = Client::new();
    client.get(url)
        .send()
        .map_err(|e| {
            eprintln!("client err, {:?}", e);
            error::PayloadError::Overflow //just a error
        })
        .and_then(|mut res| {
            res.body()
        })
        .map_err(|e| PIError::BadRequest(e.to_string()))
        .and_then(|body| {
            process_image_fut(body.to_vec())
        })
}

pub fn process_url_fut_binarydata (url: String) -> impl Future<Item = Vec<u8>, Error = PIError> {
    let client = Client::new();
    Box::new(
        client.get(url)
            .send()
            .map_err(|e| {
                eprintln!("client err, {:?}", e);
                error::PayloadError::Overflow //just a error
            })
            .and_then(|mut res| {
                res.body()
            })
            .map_err(|e| PIError::BadRequest(e.to_string()))
            .map(|body| body.to_vec())
    )
}
