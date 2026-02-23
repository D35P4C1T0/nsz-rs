use crate::error::NszError;

const UNCOMPRESSABLE_HEADER_SIZE: usize = 0x4000;

pub fn compress_nca_to_ncz_vec(data: &[u8], level: i32) -> Result<Vec<u8>, NszError> {
    if data.len() < UNCOMPRESSABLE_HEADER_SIZE {
        return Err(NszError::ContainerFormat {
            message: "NCA data too short for NCZ conversion".to_string(),
        });
    }

    let payload_size = (data.len() - UNCOMPRESSABLE_HEADER_SIZE) as u64;
    let compressed = zstd::stream::encode_all(&data[UNCOMPRESSABLE_HEADER_SIZE..], level)?;

    let mut out = Vec::with_capacity(UNCOMPRESSABLE_HEADER_SIZE + 16 + 64 + compressed.len());
    out.extend_from_slice(&data[..UNCOMPRESSABLE_HEADER_SIZE]);
    out.extend_from_slice(b"NCZSECTN");
    out.extend_from_slice(&(1u64).to_le_bytes());
    out.extend_from_slice(&(UNCOMPRESSABLE_HEADER_SIZE as u64).to_le_bytes());
    out.extend_from_slice(&payload_size.to_le_bytes());
    out.extend_from_slice(&0u64.to_le_bytes());
    out.extend_from_slice(&0u64.to_le_bytes());
    out.extend_from_slice(&[0u8; 16]);
    out.extend_from_slice(&[0u8; 16]);
    out.extend_from_slice(&compressed);
    Ok(out)
}
