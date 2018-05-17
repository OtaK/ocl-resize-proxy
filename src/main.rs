#[allow(unused_imports)]
#[macro_use] extern crate log;
extern crate env_logger;
extern crate futures;
extern crate image;
extern crate ocl;
extern crate actix_web;
extern crate actix;

mod controllers;

use futures::Future;
use std::env;
use actix_web::*;
use actix_web::http::Method;

pub type ApiResponse = Box<Future<Item=HttpResponse, Error=Error>>;

fn main() {
    env_logger::init();

    let sys = actix::System::new("ocl-resizer");

    env::set_var("RUST_LOG", "actix_web=info");

    server::new(|| App::new()
        .middleware(middleware::Logger::new("%a %t \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %Dms"))
        .resource("/resize/{uri}/{w}x{h}", |r| {
            r.method(Method::GET)
            .a(self::controllers::resize)
        })
    )
        .bind("localhost:8888").unwrap()
        .start();

    let _ = sys.run();
}
