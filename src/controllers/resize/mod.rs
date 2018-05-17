use ::ApiResponse;
use futures::future::ok as fut_ok;
use actix_web::*;

use image::{self, GenericImage};
use ocl::{self, ProQue, Image, enums::{ImageChannelOrder, ImageChannelDataType, MemObjectType}};

pub fn resize(req: HttpRequest) -> ApiResponse {
    let uri = req.match_info().get("uri").unwrap();
    let w: u32 = req.match_info().get("w").unwrap().parse::<u32>().unwrap();
    let h: u32 = req.match_info().get("h").unwrap().parse::<u32>().unwrap();

    client::ClientRequest::get(&uri)
        .finish().unwrap()
        .send()
        .map_err(Error::from)
        .and_then(|res| {
            let img = image::DynamicImage::from(res.body);
            let new_img = ocl_resize_image(img, w, h).unwrap();

            fut_ok(
                HttpResponse::Ok()
                .content_type(req.content_type())
                .body(new_img.to_rgb().into_raw())
            )
        })
        .responder()
}

fn ocl_resize_image(img: image::DynamicImage, w: u32, h: u32) -> Result<image::DynamicImage, ocl::Error> {
    let ker_path = "./ocl-resizer-kernel.cl";
    let dims = img.dimensions();

    let program = ProQue::builder()
        .src(ker_path)
        .dims(&dims)
        .build()?;

    let cl_source = Image::<u8>::builder()
        .channel_order(ImageChannelOrder::Rgb)
        .channel_data_type(ImageChannelDataType::UnormInt8)
        .image_type(MemObjectType::Image2d)
        .dims(&dims)
        .flags(ocl::flags::MEM_READ_ONLY | ocl::flags::MEM_HOST_WRITE_ONLY | ocl::flags::MEM_COPY_HOST_PTR)
        .queue(program.queue().clone())
        .copy_host_slice(&img.raw_pixels())
        .build()?;

    let mut result_unrolled: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = image::ImageBuffer::new(w, h);

    let cl_dest_unrolled = Image::<u8>::builder()
        .channel_order(ImageChannelOrder::Rgb)
        .channel_data_type(ImageChannelDataType::UnormInt8)
        .image_type(MemObjectType::Image2d)
        .dims(&dims)
        .flags(ocl::flags::MEM_WRITE_ONLY | ocl::flags::MEM_HOST_READ_ONLY | ocl::flags::MEM_COPY_HOST_PTR)
        .queue(program.queue().clone())
        .copy_host_slice(&result_unrolled)
        .build()?;

    let kernel = program.kernel_builder("resizeImage")
        .queue(program.queue().clone())
        .global_work_size(&dims)
        .arg(&cl_source)
        .arg(&cl_dest_unrolled)
        .build()?;

    kernel.enq()?;
    program.queue().finish()?;
    cl_dest_unrolled.read(&mut result_unrolled).enq()?;
    Ok(image::DynamicImage::ImageRgb8(result_unrolled))
}
