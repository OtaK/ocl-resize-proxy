use actix_web::{http::header, *};
use futures::{future::ok as fut_ok, Future};
use image::{self, GenericImage, ImageOutputFormat};
use lib::Elapsed;
use percent_encoding::percent_decode;
use std::time::Instant;
use {actors::ocl_executor, ApiResponse, ResizerState};

const BODY_LIMIT: usize = 104_857_600;

pub fn from_uri(req: HttpRequest<ResizerState>) -> ApiResponse {
    let (uri, w, h) = {
        let params = req.match_info();
        let uri: String = format!(
            "{}",
            percent_decode(params.query::<String>("uri").unwrap().as_bytes()).decode_utf8_lossy()
        );
        let w: u32 = params.query("w").unwrap();
        let h: u32 = params.query("h").unwrap();
        (uri, w, h)
    };

    client::get(&uri)
        .header(header::REFERER, "https://google.com")
        .header(header::ACCEPT, "image/jpeg")
        .header(header::USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_13_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/67.0.3396.99 Safari/537.36")
        .finish()
        .unwrap()
        .send()
        .from_err()
        .and_then(move |res| {
            let ct: String = res.content_type().into();
            let b = res.body().limit(BODY_LIMIT).wait().unwrap();
            let img = match image::load_from_memory(&b) {
                Ok(img) => img,
                Err(e) => return fut_ok(HttpResponse::BadRequest().body(format!("{}", e))),
            };

            let timer = Instant::now();
            match req.state()
                .ocl
                .send(ocl_executor::Resize { img, w, h })
                .wait()
            {
                Ok(maybe_img) => match maybe_img {
                    Ok(img) => {
                        info!("Scaled down in {}", Elapsed::from(&timer));
                        let dims = img.dimensions();
                        let mut buf = Vec::with_capacity(dims.0 as usize * dims.1 as usize);
                        match img.write_to(&mut buf, ImageOutputFormat::JPEG(100)) {
                            Ok(_) => fut_ok(HttpResponse::Ok().content_type(ct.as_str()).body(buf)),
                            Err(e) => {
                                fut_ok(HttpResponse::InternalServerError().body(format!("{}", e)))
                            }
                        }
                    }
                    Err(e) => fut_ok(HttpResponse::InternalServerError().body(format!("{}", e))),
                },
                Err(e) => fut_ok(HttpResponse::InternalServerError().body(format!("{}", e))),
            }
        })
        .responder()
}
