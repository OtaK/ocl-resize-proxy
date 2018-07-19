use actix::MailboxError;
use ocl::Error as OCLError;

#[derive(Debug, Fail)]
pub enum OCLExecutorError {
    #[fail(display = "{}", _0)]
    ActixError(MailboxError),
    #[fail(display = "{}", _0)]
    OCLError(OCLError),
    #[fail(display = "{}", _0)]
    Unknown(&'static str),
}

unsafe impl Sync for OCLExecutorError {}
unsafe impl Send for OCLExecutorError {}

impl From<&'static str> for OCLExecutorError {
    fn from(s: &'static str) -> Self {
        OCLExecutorError::Unknown(s)
    }
}

impl From<MailboxError> for OCLExecutorError {
    fn from(e: MailboxError) -> Self {
        OCLExecutorError::ActixError(e)
    }
}

impl From<OCLError> for OCLExecutorError {
    fn from(e: OCLError) -> Self {
        OCLExecutorError::OCLError(e)
    }
}
