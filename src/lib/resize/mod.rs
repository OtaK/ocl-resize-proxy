pub static KERNEL_SRC: &'static str = include_str!("ocl-resizer-kernel.cl");

use image::{self, GenericImage};
use ocl::{
    self, enums::{ImageChannelDataType, ImageChannelOrder, MemObjectType}, Image, ProQue,
};

#[macro_export]
macro_rules! proque {
    ($dims:expr) => {{
        use lib::KERNEL_SRC;
        use ocl::ProQue;
        ProQue::builder().src(KERNEL_SRC).dims(&$dims).build()
    }};
}

#[allow(dead_code)]
pub fn ocl_resize_image(
    img: &image::DynamicImage,
    w: u32,
    h: u32,
) -> ocl::Result<image::DynamicImage> {
    let dims = img.dimensions();
    let mut program = proque!(dims)?;
    ocl_resize_image_with_proque(&mut program, img, w, h)
}

pub fn ocl_resize_image_with_proque(
    program: &mut ProQue,
    img: &image::DynamicImage,
    w: u32,
    h: u32,
) -> ocl::Result<image::DynamicImage> {
    let dims = img.dimensions();
    let (new_w, new_h) = resize_dimensions(dims.0, dims.1, w, h, false);
    program.set_dims(dims);
    let img_buf = img.to_rgba();
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

    let mut result_unrolled: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
        image::ImageBuffer::new(new_w, new_h);

    let cl_dest_unrolled = Image::<u8>::builder()
        .channel_order(ImageChannelOrder::Rgba)
        .channel_data_type(ImageChannelDataType::UnormInt8)
        .image_type(MemObjectType::Image2d)
        .dims(result_unrolled.dimensions())
        .flags(ocl::flags::MEM_WRITE_ONLY)
        .queue(program.queue().clone())
        .copy_host_slice(&result_unrolled)
        .build()?;

    let work_size = (
        ::std::cmp::max(dims.0, new_w),
        ::std::cmp::max(dims.1, new_h),
    );

    let kernel = program
        .kernel_builder("resizeImage")
        .queue(program.queue().clone())
        .global_work_size(work_size)
        .arg(&cl_source)
        .arg(&cl_dest_unrolled)
        .build()?;

    unsafe {
        kernel.enq()?;
    }
    program.queue().finish()?;
    cl_dest_unrolled.read(&mut result_unrolled).enq()?;
    Ok(image::DynamicImage::ImageRgba8(result_unrolled))
}

fn resize_dimensions(width: u32, height: u32, nwidth: u32, nheight: u32, fill: bool) -> (u32, u32) {
    let ratio = width as u64 * nheight as u64;
    let nratio = nwidth as u64 * height as u64;

    let use_width = if fill {
        nratio > ratio
    } else {
        nratio <= ratio
    };

    let intermediate = if use_width {
        height as u64 * nwidth as u64 / width as u64
    } else {
        width as u64 * nheight as u64 / height as u64
    };

    if use_width {
        if intermediate <= ::std::u32::MAX as u64 {
            (nwidth, intermediate as u32)
        } else {
            (
                (nwidth as u64 * ::std::u32::MAX as u64 / intermediate) as u32,
                ::std::u32::MAX,
            )
        }
    } else {
        if intermediate <= ::std::u32::MAX as u64 {
            (intermediate as u32, nheight)
        } else {
            (
                ::std::u32::MAX,
                (nheight as u64 * ::std::u32::MAX as u64 / intermediate) as u32,
            )
        }
    }
}
