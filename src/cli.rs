#[allow(unused_imports)]
#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate image;
extern crate ocl;

mod lib;
use self::lib::*;

use clap::App;
use image::GenericImage;
use std::fmt;
use std::fs::File;
use std::time::{Duration, Instant};

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

fn main() {
    env_logger::init();

    let matches = App::new("ocl-image-resizer")
        .version("0.0.1")
        .about("Resizes images using OpenCL")
        .author("Mathieu Amiot <amiot.mathieu@gmail.com>")
        .args_from_usage(
            "-w, --width=[WIDTH]    'Sets the resize width'
            <FILE>                  'Sets the input file to use'",
        )
        .get_matches();

    let file = matches.value_of("FILE").unwrap();
    let w: f64 = matches.value_of("width").unwrap_or("400").parse().unwrap();

    let img = image::open(file).unwrap();

    let dims = img.dimensions();
    let h = dims.1 as f64 * (w as f64 / dims.0 as f64);
    info!("resizing to {}x{} from {}x{}", w, h, dims.0, dims.1);
    let timer = Instant::now();
    let new_img = ocl_resize_image(&img, w as u32, h as u32).unwrap();
    info!("Scaled down in {}", Elapsed::from(&timer));
    info!("new image dims {:?}", new_img.dimensions());

    let mut f = File::create(&format!("resized-{}", file)).expect("File creation failed");
    let mut encoder = image::jpeg::JPEGEncoder::new_with_quality(&mut f, 100);
    let _ = encoder.encode(&new_img.raw_pixels(), w as u32, h as u32, new_img.color());
    info!("done!");
}
