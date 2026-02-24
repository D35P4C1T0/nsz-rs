use std::io::Write;

use aes::Aes128;
use ctr::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
use zstd::stream::write::Encoder;

use crate::container::nca::{NcaCompressionPlan, NcaEncryptionSection};
use crate::error::NszError;

const UNCOMPRESSABLE_HEADER_SIZE: usize = 0x4000;
const CHUNK_SIZE: usize = 0x1000000;
type AesCtr = ctr::Ctr128BE<Aes128>;

#[derive(Debug, Clone, Copy)]
struct PayloadPart {
    offset: u64,
    size: u64,
    crypto_type: u64,
    crypto_key: [u8; 16],
    crypto_counter: [u8; 16],
    encrypted: bool,
}

pub fn compress_nca_to_ncz_vec(data: &[u8], level: i32) -> Result<Vec<u8>, NszError> {
    compress_nca_to_ncz_vec_with_plan(data, level, false, 1, None)
}

pub fn compress_nca_to_ncz_vec_with_plan(
    data: &[u8],
    level: i32,
    long_distance_mode: bool,
    threads: i32,
    plan: Option<&NcaCompressionPlan>,
) -> Result<Vec<u8>, NszError> {
    if data.len() < UNCOMPRESSABLE_HEADER_SIZE {
        return Err(NszError::ContainerFormat {
            message: "NCA data too short for NCZ conversion".to_string(),
        });
    }

    if let Some(plan) = plan {
        return compress_with_plan(data, level, long_distance_mode, threads, plan);
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

pub fn compress_nca_to_ncz_block_vec_with_plan(
    data: &[u8],
    level: i32,
    long_distance_mode: bool,
    block_size_exponent: u8,
    plan: Option<&NcaCompressionPlan>,
) -> Result<Vec<u8>, NszError> {
    if data.len() < UNCOMPRESSABLE_HEADER_SIZE {
        return Err(NszError::ContainerFormat {
            message: "NCA data too short for NCZ conversion".to_string(),
        });
    }

    if let Some(plan) = plan {
        return compress_block_with_sections(
            data,
            level,
            long_distance_mode,
            block_size_exponent,
            &plan.sections,
            &build_parts(plan),
        );
    }

    let section = NcaEncryptionSection {
        offset: UNCOMPRESSABLE_HEADER_SIZE as u64,
        size: (data.len() - UNCOMPRESSABLE_HEADER_SIZE) as u64,
        crypto_type: 0,
        crypto_key: [0u8; 16],
        crypto_counter: [0u8; 16],
    };
    let part = PayloadPart {
        offset: UNCOMPRESSABLE_HEADER_SIZE as u64,
        size: section.size,
        crypto_type: 0,
        crypto_key: [0u8; 16],
        crypto_counter: [0u8; 16],
        encrypted: false,
    };
    compress_block_with_sections(
        data,
        level,
        long_distance_mode,
        block_size_exponent,
        &[section],
        &[part],
    )
}

fn compress_with_plan(
    data: &[u8],
    level: i32,
    long_distance_mode: bool,
    threads: i32,
    plan: &NcaCompressionPlan,
) -> Result<Vec<u8>, NszError> {
    let mut output = Vec::with_capacity(data.len());
    output.extend_from_slice(&data[..UNCOMPRESSABLE_HEADER_SIZE]);
    output.extend_from_slice(b"NCZSECTN");
    output.extend_from_slice(&(plan.sections.len() as u64).to_le_bytes());
    for section in &plan.sections {
        output.extend_from_slice(&section.offset.to_le_bytes());
        output.extend_from_slice(&section.size.to_le_bytes());
        output.extend_from_slice(&section.crypto_type.to_le_bytes());
        output.extend_from_slice(&0u64.to_le_bytes());
        output.extend_from_slice(&section.crypto_key);
        output.extend_from_slice(&section.crypto_counter);
    }

    let parts = build_parts(plan);

    let mut encoder = Encoder::new(output, level)?;
    encoder.long_distance_matching(long_distance_mode)?;
    if threads > 1 {
        encoder.multithread(threads as u32)?;
    }

    let mut scratch = Vec::with_capacity(CHUNK_SIZE);
    for part in &parts {
        let mut cipher =
            (part.encrypted && matches!(part.crypto_type, 3 | 4)).then(|| {
                init_aes_ctr(&part.crypto_key, &part.crypto_counter, part.offset as u128)
            });
        let mut processed = 0u64;
        while processed < part.size {
            let to_read = (part.size - processed).min(CHUNK_SIZE as u64) as usize;
            let start = part.offset.saturating_add(processed) as usize;
            let end = start.saturating_add(to_read);
            if end > data.len() {
                return Err(NszError::ContainerFormat {
                    message: "NCZ compression part exceeds source bounds".to_string(),
                });
            }

            if let Some(cipher) = cipher.as_mut() {
                scratch.clear();
                scratch.extend_from_slice(&data[start..end]);
                cipher.apply_keystream(&mut scratch);
                encoder.write_all(&scratch)?;
            } else {
                encoder.write_all(&data[start..end])?;
            }
            processed = processed.saturating_add(to_read as u64);
        }
    }

    Ok(encoder.finish()?)
}

fn build_parts(plan: &NcaCompressionPlan) -> Vec<PayloadPart> {
    let mut parts = Vec::with_capacity(plan.sections.len() + 1);
    if plan.offset_first_section > UNCOMPRESSABLE_HEADER_SIZE as u64 {
        parts.push(PayloadPart {
            offset: UNCOMPRESSABLE_HEADER_SIZE as u64,
            size: plan.offset_first_section - UNCOMPRESSABLE_HEADER_SIZE as u64,
            crypto_type: 0,
            crypto_key: [0u8; 16],
            crypto_counter: [0u8; 16],
            encrypted: false,
        });
    }
    for section in &plan.sections {
        parts.push(PayloadPart {
            offset: section.offset,
            size: section.size,
            crypto_type: section.crypto_type,
            crypto_key: section.crypto_key,
            crypto_counter: section.crypto_counter,
            encrypted: true,
        });
    }

    let mut skip =
        UNCOMPRESSABLE_HEADER_SIZE.saturating_sub(plan.offset_first_section as usize) as u64;
    for part in &mut parts {
        if skip == 0 {
            break;
        }
        let consumed = skip.min(part.size);
        part.offset = part.offset.saturating_add(consumed);
        part.size = part.size.saturating_sub(consumed);
        skip -= consumed;
    }
    parts.retain(|part| part.size > 0);
    parts
}

fn compress_block_with_sections(
    data: &[u8],
    level: i32,
    long_distance_mode: bool,
    block_size_exponent: u8,
    sections: &[NcaEncryptionSection],
    parts: &[PayloadPart],
) -> Result<Vec<u8>, NszError> {
    if !(14..=32).contains(&block_size_exponent) {
        return Err(NszError::ContainerFormat {
            message: "NCZBLOCK block size exponent out of range".to_string(),
        });
    }
    let block_size = 1usize << block_size_exponent;

    let mut out = Vec::with_capacity(data.len());
    out.extend_from_slice(&data[..UNCOMPRESSABLE_HEADER_SIZE]);
    out.extend_from_slice(b"NCZSECTN");
    out.extend_from_slice(&(sections.len() as u64).to_le_bytes());
    for section in sections {
        out.extend_from_slice(&section.offset.to_le_bytes());
        out.extend_from_slice(&section.size.to_le_bytes());
        out.extend_from_slice(&section.crypto_type.to_le_bytes());
        out.extend_from_slice(&0u64.to_le_bytes());
        out.extend_from_slice(&section.crypto_key);
        out.extend_from_slice(&section.crypto_counter);
    }

    let decompressed_size = parts
        .iter()
        .fold(0u64, |acc, part| acc.saturating_add(part.size));
    let mut block_payloads = Vec::new();
    let mut block_sizes = Vec::new();
    let mut pending = Vec::with_capacity(block_size);
    let mut scratch = Vec::with_capacity(CHUNK_SIZE);

    for part in parts {
        let mut cipher =
            (part.encrypted && matches!(part.crypto_type, 3 | 4)).then(|| {
                init_aes_ctr(&part.crypto_key, &part.crypto_counter, part.offset as u128)
            });
        let mut processed = 0u64;
        while processed < part.size {
            let to_read = (part.size - processed).min(CHUNK_SIZE as u64) as usize;
            let start = part.offset.saturating_add(processed) as usize;
            let end = start.saturating_add(to_read);
            if end > data.len() {
                return Err(NszError::ContainerFormat {
                    message: "NCZ compression part exceeds source bounds".to_string(),
                });
            }

            if let Some(cipher) = cipher.as_mut() {
                scratch.clear();
                scratch.extend_from_slice(&data[start..end]);
                cipher.apply_keystream(&mut scratch);
                push_payload_to_blocks(
                    &scratch,
                    block_size,
                    level,
                    long_distance_mode,
                    &mut pending,
                    &mut block_sizes,
                    &mut block_payloads,
                )?;
            } else {
                push_payload_to_blocks(
                    &data[start..end],
                    block_size,
                    level,
                    long_distance_mode,
                    &mut pending,
                    &mut block_sizes,
                    &mut block_payloads,
                )?;
            }

            processed = processed.saturating_add(to_read as u64);
        }
    }

    if !pending.is_empty() {
        let block = encode_block_payload(&pending, block_size, level, long_distance_mode)?;
        let block_size_u32 = u32::try_from(block.len()).map_err(|_| NszError::ContainerFormat {
            message: "NCZBLOCK compressed block too large".to_string(),
        })?;
        block_sizes.push(block_size_u32);
        block_payloads.push(block);
    }

    let block_count = u32::try_from(block_sizes.len()).map_err(|_| NszError::ContainerFormat {
        message: "NCZBLOCK block count overflow".to_string(),
    })?;

    out.extend_from_slice(b"NCZBLOCK");
    out.extend_from_slice(&[0x02, 0x01, 0x00, block_size_exponent]);
    out.extend_from_slice(&block_count.to_le_bytes());
    out.extend_from_slice(&decompressed_size.to_le_bytes());
    for size in &block_sizes {
        out.extend_from_slice(&size.to_le_bytes());
    }
    for block in block_payloads {
        out.extend_from_slice(&block);
    }
    Ok(out)
}

fn encode_block_payload(
    payload: &[u8],
    block_size: usize,
    level: i32,
    long_distance_mode: bool,
) -> Result<Vec<u8>, NszError> {
    if level == 0 && payload.len() == block_size {
        return Ok(payload.to_vec());
    }

    let compressed = if long_distance_mode {
        let mut encoder = Encoder::new(Vec::new(), level)?;
        encoder.long_distance_matching(true)?;
        encoder.write_all(payload)?;
        encoder.finish()?
    } else {
        zstd::bulk::compress(payload, level)?
    };
    if compressed.len() < payload.len() {
        Ok(compressed)
    } else {
        Ok(payload.to_vec())
    }
}

fn push_payload_to_blocks(
    payload: &[u8],
    block_size: usize,
    level: i32,
    long_distance_mode: bool,
    pending: &mut Vec<u8>,
    block_sizes: &mut Vec<u32>,
    block_payloads: &mut Vec<Vec<u8>>,
) -> Result<(), NszError> {
    let mut cursor = 0usize;
    while cursor < payload.len() {
        let take = (block_size - pending.len()).min(payload.len() - cursor);
        pending.extend_from_slice(&payload[cursor..cursor + take]);
        cursor += take;

        if pending.len() == block_size {
            let block = encode_block_payload(pending, block_size, level, long_distance_mode)?;
            let block_size_u32 =
                u32::try_from(block.len()).map_err(|_| NszError::ContainerFormat {
                    message: "NCZBLOCK compressed block too large".to_string(),
                })?;
            block_sizes.push(block_size_u32);
            block_payloads.push(block);
            pending.clear();
        }
    }
    Ok(())
}

fn init_aes_ctr(key: &[u8; 16], counter: &[u8; 16], offset: u128) -> AesCtr {
    let mut cipher = AesCtr::new(key.into(), counter.into());
    cipher.seek(offset);
    cipher
}

#[cfg(test)]
mod tests {
    use super::{init_aes_ctr, AesCtr};
    use ctr::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};

    #[test]
    fn ctr_streaming_matches_seek_per_chunk() {
        let key = [0x3Au8; 16];
        let counter = [0xC1u8; 16];
        let offset = 0x2345u128;
        let payload: Vec<u8> = (0u32..16384).map(|idx| (idx % 251) as u8).collect();
        let mut stream_output = payload.clone();
        let mut stream_cipher = init_aes_ctr(&key, &counter, offset);
        stream_cipher.apply_keystream(&mut stream_output);

        let mut chunked_output = payload;
        let mut cursor = 0usize;
        let chunk_size = 257usize;
        while cursor < chunked_output.len() {
            let end = (cursor + chunk_size).min(chunked_output.len());
            let mut chunk_cipher = AesCtr::new((&key).into(), (&counter).into());
            chunk_cipher.seek(offset + cursor as u128);
            chunk_cipher.apply_keystream(&mut chunked_output[cursor..end]);
            cursor = end;
        }

        assert_eq!(stream_output, chunked_output);
    }
}
