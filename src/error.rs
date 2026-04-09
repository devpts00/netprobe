use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetprobeError {
    #[error("unexpected: {0}")]
    Unexpected(&'static str),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("nul: {0}")]
    Nul(#[from] std::ffi::NulError),
}
