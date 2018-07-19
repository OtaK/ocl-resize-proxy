use actix::prelude::*;
use image;
use lib::ocl_resize_image_with_proque;
use ocl::ProQue;

mod error;
pub use self::error::*;

pub type OCLExecutorResult<T> = ::std::result::Result<T, OCLExecutorError>;

pub struct OCLExecutorSync(ProQue);
impl OCLExecutorSync {
    fn new(k: ProQue) -> Self {
        OCLExecutorSync(k)
    }

    pub fn start<F>(threads: usize, client_factory: F) -> Addr<Syn, Self>
    where
        F: Fn() -> ProQue + Send + Sync + 'static,
    {
        SyncArbiter::start(threads, move || Self::new(client_factory()))
    }
}

impl Actor for OCLExecutorSync {
    type Context = SyncContext<Self>;
}

pub struct Resize {
    pub img: image::DynamicImage,
    pub w: u32,
    pub h: u32,
}

impl Message for Resize {
    type Result = OCLExecutorResult<image::DynamicImage>;
}

impl Handler<Resize> for OCLExecutorSync {
    type Result = OCLExecutorResult<image::DynamicImage>;

    fn handle(&mut self, cmd: Resize, _: &mut Self::Context) -> Self::Result {
        match ocl_resize_image_with_proque(&mut self.0, &cmd.img, cmd.w, cmd.h) {
            Ok(img) => Ok(img),
            Err(e) => Err(OCLExecutorError::OCLError(e)),
        }
    }
}
