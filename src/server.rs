#[macro_use]
extern crate log;
extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate image;
extern crate ocl;
#[macro_use]
extern crate failure;
extern crate percent_encoding;

mod actors;
mod controllers;
#[macro_use]
mod lib;

use self::actors::ocl_executor::*;

use actix::*;
use actix_web::http::Method;
use actix_web::*;
use futures::Future;
use std::env;

#[derive(Clone)]
pub struct ResizerState {
    ocl: Addr<Syn, OCLExecutorSync>,
}

pub type ApiResponse = Box<Future<Item = HttpResponse, Error = Error>>;

fn main() {
    env_logger::init();

    let sys = actix::System::new("ocl-resizer");

    env::set_var("RUST_LOG", "actix_web=info");

    let ocl_addr: Addr<Syn, OCLExecutorSync> = OCLExecutorSync::start(4, || proque!().unwrap());

    server::new(move || {
        App::with_state(ResizerState {
            ocl: ocl_addr.clone(),
        }).middleware(middleware::Logger::new(
            "%a %t \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %Dms",
        ))
            .resource("/resize/{uri}/{w}x{h}", |r| {
                r.method(Method::GET)
                    .with_async(self::controllers::resize::from_uri)
            })
    }).bind("localhost:8888")
        .unwrap()
        .start();

    let _ = sys.run();
}
