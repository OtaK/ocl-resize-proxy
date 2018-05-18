use std::fs::File;
use std::io::Read;
/*use ::ApiResponse;
use futures::future::ok as fut_ok;
use futures::Future;
use futures::Stream;

use actix_web::*;*/

use image::{self, GenericImage};
use ocl::{self, ProQue, Image, enums::{ImageChannelOrder, ImageChannelDataType, MemObjectType}};


pub fn ocl_resize_image(img: image::DynamicImage, w: u32, h: u32) -> Result<image::DynamicImage, ocl::Error> {
    let dims = img.dimensions();
    let img_buf = img.to_rgba();

    let mut ker_file = File::open("./src/controllers/resize/ocl-resizer-kernel.cl").expect("Kernel source not present");
    let mut ker_src = String::new();
    ker_file.read_to_string(&mut ker_src).expect("Could not read the file");

    let program = ProQue::builder()
        .src(ker_src)
        .dims(&dims)
        .build()?;

    let img_pixels = img_buf.clone().into_vec();

    let cl_source = Image::<u8>::builder()
        .channel_order(ImageChannelOrder::Rgba)
        .channel_data_type(ImageChannelDataType::UnormInt8)
        .image_type(MemObjectType::Image2d)
        .dims(&dims)
        .flags(ocl::flags::MEM_READ_ONLY)
        .queue(program.queue().clone())
        .copy_host_slice(&img_pixels)
        .build()?;

    let mut result_unrolled: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::new(w, h);

    let cl_dest_unrolled = Image::<u8>::builder()
        .channel_order(ImageChannelOrder::Rgba)
        .channel_data_type(ImageChannelDataType::UnormInt8)
        .image_type(MemObjectType::Image2d)
        .dims(result_unrolled.dimensions())
        .flags(ocl::flags::MEM_WRITE_ONLY)
        .queue(program.queue().clone())
        .copy_host_slice(&result_unrolled)
        .build()?;

    let kernel = program.kernel_builder("resizeImage")
        .queue(program.queue().clone())
        .global_work_size(&dims)
        .arg(&cl_source)
        .arg(&cl_dest_unrolled)
        .build()?;

    unsafe { kernel.enq()?; }
    program.queue().finish()?;
    cl_dest_unrolled.read(&mut result_unrolled).enq()?;
    Ok(image::DynamicImage::ImageRgba8(result_unrolled))
}

/*pub fn resize(req: HttpRequest) -> ApiResponse {
    let uri = req.match_info().get("uri").unwrap();
    let w: u32 = req.match_info().get("w").unwrap().parse().unwrap();
    let h: u32 = req.match_info().get("h").unwrap().parse().unwrap();

    client::ClientRequest::get(&uri)
        .finish().unwrap()
        .send()
        .map_err(Error::from)
        .and_then(|res| {
            res.body()
            .and_then(|b| {
                let img = match image::load_from_memory(&b.take()) {
                    Ok(img) => img,
                    Err(_) => { return fut_ok(HttpResponse::BadRequest().finish()) }
                };
                let new_img = ocl_resize_image(img, w, h).unwrap();

                fut_ok(
                    HttpResponse::Ok()
                    .content_type(req.content_type())
                    .body(new_img.to_rgb().into_raw())
                )
            })
        })
        .responder()
}*/

