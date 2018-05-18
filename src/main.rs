#[allow(unused_imports)]
#[macro_use] extern crate log;
extern crate env_logger;
//extern crate futures;
extern crate image;
extern crate ocl;
//extern crate actix_web;
//extern crate actix;
extern crate clap;

mod controllers;

//use futures::Future;
//use std::env;
//use actix_web::*;
//use actix_web::http::Method;
use clap::App;
use image::GenericImage;
use std::fmt;
use std::time::{Duration, Instant};
use std::fs::File;


struct Elapsed(Duration);

impl Elapsed {
    fn from(start: &Instant) -> Self {
        Elapsed(start.elapsed())
    }
}

impl fmt::Display for Elapsed {
    fn fmt(&self, out: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match (self.0.as_secs(), self.0.subsec_nanos()) {
            (0, n) if n < 1000 => write!(out, "{} ns", n),
            (0, n) if n < 1000_000 => write!(out, "{} µs", n / 1000),
            (0, n) => write!(out, "{} ms", n / 1000_000),
            (s, n) if s < 10 => write!(out, "{}.{:02} s", s, n / 10_000_000),
            (s, _) => write!(out, "{} s", s),
        }
    }
}

fn img_formats() -> ocl::Result<()> {
    use ocl::{Platform, Device, Context, Image};
    use ocl::enums::MemObjectType;

    for (p_idx, platform) in Platform::list().into_iter().enumerate() {
        for (d_idx, device) in Device::list_all(&platform)?.into_iter().enumerate() {
            println!("Platform [{}]: {}", p_idx, platform.name()?);
            println!("Device [{}]: {} {}", d_idx, device.vendor()?, device.name()?);

            let context = Context::builder().platform(platform).devices(device).build()?;

            let sup_img_formats = Image::<u8>::supported_formats(&context,
                ocl::flags::MEM_READ_WRITE,
                MemObjectType::Image2d
            )?;

            println!("Image Formats: {:#?}.", sup_img_formats);
        }
    }

    Ok(())
}

//pub type ApiResponse = Box<Future<Item=HttpResponse, Error=Error>>;

fn main() {
    env_logger::init();

    /*let sys = actix::System::new("ocl-resizer");

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

    let _ = sys.run();*/

    let matches = App::new("ocl-image-resizer")
        .version("0.0.1")
        .about("Resizes images using OpenCL")
        .author("Mathieu Amiot <amiot.mathieu@gmail.com>")
        .args_from_usage(
            "-w, --width=[WIDTH]    'Sets the resize width'
            <FILE>                  'Sets the input file to use'")
        .get_matches();


    //let _ = img_formats();

    let file = matches.value_of("FILE").unwrap();
    let w: f64 = matches.value_of("width").unwrap_or("400").parse().unwrap();

    let img = image::open(file).unwrap();

    let dims = img.dimensions();
    let h = dims.1 as f64 * (w as f64 / dims.0 as f64);
    println!("resizing to {}x{} from {}x{}", w, h, dims.0, dims.1);
    let timer = Instant::now();
    let new_img = self::controllers::ocl_resize_image(img, w as u32, h as u32).unwrap();
    let elapsed = Elapsed::from(&timer);
    println!("new image dims {:?}", new_img.dimensions());
    println!("Scaled down in {}", elapsed);

    let mut f = File::create(&format!("resized-{}", file)).expect("File creation failed");
    let mut encoder = image::jpeg::JPEGEncoder::new_with_quality(&mut f, 100);
    let _ = encoder.encode(&new_img.raw_pixels(), w as u32, h as u32, new_img.color());

    //let _ = new_img.save(&format!("resized-{}", file)).unwrap();
    println!("done!");

}
