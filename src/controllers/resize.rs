use actix_web::*;
use futures::{future::ok as fut_ok, Future};
use image;
use {actors::ocl_executor, ApiResponse, ResizerState};

pub fn from_uri(req: HttpRequest<ResizerState>) -> ApiResponse {
    let (uri, w, h) = {
        let params = req.match_info();
        let uri: String = params.query("uri").unwrap();
        let w: u32 = params.query("w").unwrap();
        let h: u32 = params.query("h").unwrap();
        (uri, w, h)
    };

    client::ClientRequest::get(&uri)
        .finish()
        .unwrap()
        .send()
        .from_err()
        .and_then(move |res| {
            let b = res.body().wait().unwrap();
            let img = match image::load_from_memory(&b) {
                Ok(img) => img,
                Err(_) => return fut_ok(HttpResponse::BadRequest().finish()),
            };

            match req.state()
                .ocl
                .send(ocl_executor::Resize { img, w, h })
                .wait()
            {
                Ok(maybe_img) => match maybe_img {
                    Ok(img) => fut_ok(
                        HttpResponse::Ok()
                            .content_type(req.content_type())
                            .body(img.to_rgb().into_raw()),
                    ),
                    Err(_) => fut_ok(HttpResponse::InternalServerError().finish()),
                },
                Err(_) => fut_ok(HttpResponse::InternalServerError().finish()),
            }
        })
        .responder()
}
