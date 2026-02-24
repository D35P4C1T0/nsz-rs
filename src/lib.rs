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

pub fn compress(request: &CompressRequest) -> Result<OperationReport, NszError> {
    ops::compress::run(request)
}

pub fn decompress(request: &DecompressRequest) -> Result<OperationReport, NszError> {
    ops::decompress::run(request)
}

pub fn verify(request: &VerifyRequest) -> Result<VerifyReport, NszError> {
    ops::verify::run(request)
}

pub fn extract(request: &ExtractRequest) -> Result<OperationReport, NszError> {
    ops::extract::run(request)
}

pub fn create(request: &CreateRequest) -> Result<OperationReport, NszError> {
    ops::create::run(request)
}

pub fn titlekeys(request: &TitleKeysRequest) -> Result<OperationReport, NszError> {
    ops::titlekeys::run(request)
}

pub fn undupe(request: &UndupeRequest) -> Result<OperationReport, NszError> {
    ops::undupe::run(request)
}
