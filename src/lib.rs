pub mod config;
pub mod container;
pub mod crypto;
pub mod error;
pub mod fs_ops;
pub mod ncz;
pub mod ops;
pub mod parity;

pub use config::{
    CompressRequest, CreateRequest, DecompressRequest, ExtractRequest, TitleKeysRequest,
    UndupeRequest, VerifyRequest,
};
pub use error::NszError;
pub use ops::{OperationReport, VerifyReport};

pub fn compress(_: &CompressRequest) -> Result<OperationReport, NszError> {
    Err(NszError::NotImplemented("compress"))
}

pub fn decompress(request: &DecompressRequest) -> Result<OperationReport, NszError> {
    ops::decompress::run(request)
}

pub fn verify(request: &VerifyRequest) -> Result<VerifyReport, NszError> {
    ops::verify::run(request)
}

pub fn extract(_: &ExtractRequest) -> Result<OperationReport, NszError> {
    Err(NszError::NotImplemented("extract"))
}

pub fn create(_: &CreateRequest) -> Result<OperationReport, NszError> {
    Err(NszError::NotImplemented("create"))
}

pub fn titlekeys(_: &TitleKeysRequest) -> Result<OperationReport, NszError> {
    Err(NszError::NotImplemented("titlekeys"))
}

pub fn undupe(_: &UndupeRequest) -> Result<OperationReport, NszError> {
    Err(NszError::NotImplemented("undupe"))
}
