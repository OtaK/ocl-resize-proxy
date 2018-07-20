#![feature(test)]
extern crate image;
extern crate ocl;
extern crate test;

#[macro_use]
mod lib;

fn main() {}

#[cfg(test)]
mod bench {
    use image;
    use lib::*;
    use test::Bencher;

    #[bench]
    fn bench_ocl_huge(b: &mut Bencher) {
        let mut proque = proque!().unwrap();
        let img = image::open("./samples/sample_huge.jpg").unwrap();
        let w = 350;
        let h = 350;

        b.iter(move || ocl_resize_image_with_proque(&mut proque, &img, w, h));
    }

    #[bench]
    fn bench_ocl_dslr(b: &mut Bencher) {
        let mut proque = proque!().unwrap();
        let img = image::open("./samples/sample_dslr.jpg").unwrap();
        let w = 350;
        let h = 350;

        b.iter(move || ocl_resize_image_with_proque(&mut proque, &img, w, h));
    }
}
