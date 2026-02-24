use crate::error::NszError;
use aes::Aes128;
use ctr::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};

const UNCOMPRESSABLE_HEADER_SIZE: usize = 0x4000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NczSection {
    /// Section offset in the decompressed NCA payload.
    pub offset: u64,
    /// Section byte size.
    pub size: u64,
    /// Section crypto mode identifier.
    pub crypto_type: u64,
    /// Section AES key material.
    pub crypto_key: [u8; 16],
    /// Section AES counter bytes.
    pub crypto_counter: [u8; 16],
}

/// Returns the decompressed NCA size encoded by an NCZ payload.
pub fn decompressed_nca_size_from_bytes(data: &[u8]) -> Result<u64, NszError> {
    let (sections, _) = parse_sections_with_end(data)?;
    let payload_size = payload_size_from_sections(&sections)?;
    (UNCOMPRESSABLE_HEADER_SIZE as u64)
        .checked_add(payload_size)
        .ok_or_else(|| NszError::ContainerFormat {
            message: "NCZ decompressed size overflow".to_string(),
        })
}

/// Decompresses an NCZ image back into raw NCA bytes.
pub fn decompress_ncz_to_vec(data: &[u8]) -> Result<Vec<u8>, NszError> {
    let (sections, stream_offset) = parse_sections_with_end(data)?;

    if data.len() < stream_offset {
        return Err(NszError::ContainerFormat {
            message: "NCZ stream offset outside input".to_string(),
        });
    }

    let decompressed = decode_ncz_stream(data, stream_offset)?;

    let payload_size = payload_size_from_sections(&sections)?;
    let leading_gap = leading_gap_from_sections(&sections)?;
    let output_capacity_u64 = (UNCOMPRESSABLE_HEADER_SIZE as u64)
        .checked_add(payload_size)
        .and_then(|value| value.checked_add(leading_gap))
        .ok_or_else(|| NszError::ContainerFormat {
            message: "NCZ decompressed output size overflow".to_string(),
        })?;
    let output_capacity =
        usize::try_from(output_capacity_u64).map_err(|_| NszError::ContainerFormat {
            message: "NCZ decompressed output size exceeds platform limits".to_string(),
        })?;

    let mut output = Vec::with_capacity(output_capacity);
    output.extend_from_slice(&data[..UNCOMPRESSABLE_HEADER_SIZE]);

    let mut read_cursor = 0usize;
    if leading_gap > 0 {
        let gap = usize::try_from(leading_gap).map_err(|_| NszError::ContainerFormat {
            message: "NCZ leading gap exceeds platform limits".to_string(),
        })?;
        if decompressed.len() < gap {
            return Err(NszError::ContainerFormat {
                message: "NCZ stream shorter than leading gap".to_string(),
            });
        }
        output.extend_from_slice(&decompressed[..gap]);
        read_cursor += gap;
    }

    for section in &sections {
        let size = section.size as usize;
        let end = read_cursor + size;
        if decompressed.len() < end {
            return Err(NszError::ContainerFormat {
                message: "NCZ stream shorter than declared sections".to_string(),
            });
        }
        let write_start = output.len();
        output.extend_from_slice(&decompressed[read_cursor..end]);
        if matches!(section.crypto_type, 3 | 4) {
            apply_aes_ctr(
                &mut output[write_start..],
                &section.crypto_key,
                &section.crypto_counter,
                u128::from(section.offset),
            );
        }
        read_cursor = end;
    }

    Ok(output)
}

/// Parses the NCZ section table without decoding payload bytes.
pub fn parse_sections(data: &[u8]) -> Result<Vec<NczSection>, NszError> {
    let (sections, _) = parse_sections_with_end(data)?;
    Ok(sections)
}

fn parse_sections_with_end(data: &[u8]) -> Result<(Vec<NczSection>, usize), NszError> {
    if data.len() < UNCOMPRESSABLE_HEADER_SIZE + 16 {
        return Err(NszError::ContainerFormat {
            message: "NCZ data too short for section header".to_string(),
        });
    }

    let mut cursor = UNCOMPRESSABLE_HEADER_SIZE;
    if &data[cursor..cursor + 8] != b"NCZSECTN" {
        return Err(NszError::ContainerFormat {
            message: "NCZ section magic mismatch".to_string(),
        });
    }
    cursor += 8;

    let section_count = u64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap()) as usize;
    cursor += 8;

    let needed = cursor + section_count * 64;
    if data.len() < needed {
        return Err(NszError::ContainerFormat {
            message: "NCZ section data truncated".to_string(),
        });
    }

    let mut sections = Vec::with_capacity(section_count);
    for _ in 0..section_count {
        let offset = u64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap());
        let size = u64::from_le_bytes(data[cursor + 8..cursor + 16].try_into().unwrap());
        let crypto_type = u64::from_le_bytes(data[cursor + 16..cursor + 24].try_into().unwrap());
        let crypto_key: [u8; 16] = data[cursor + 32..cursor + 48].try_into().unwrap();
        let crypto_counter: [u8; 16] = data[cursor + 48..cursor + 64].try_into().unwrap();
        sections.push(NczSection {
            offset,
            size,
            crypto_type,
            crypto_key,
            crypto_counter,
        });
        cursor += 64;
    }

    Ok((sections, cursor))
}

fn payload_size_from_sections(sections: &[NczSection]) -> Result<u64, NszError> {
    let mut total = 0u64;
    for section in sections {
        total = total
            .checked_add(section.size)
            .ok_or_else(|| NszError::ContainerFormat {
                message: "NCZ section payload size overflow".to_string(),
            })?;
    }
    Ok(total)
}

fn leading_gap_from_sections(sections: &[NczSection]) -> Result<u64, NszError> {
    let Some(first) = sections.first() else {
        return Ok(0);
    };
    if first.offset <= UNCOMPRESSABLE_HEADER_SIZE as u64 {
        return Ok(0);
    }
    first
        .offset
        .checked_sub(UNCOMPRESSABLE_HEADER_SIZE as u64)
        .ok_or_else(|| NszError::ContainerFormat {
            message: "NCZ leading gap underflow".to_string(),
        })
}

fn decode_ncz_stream(data: &[u8], stream_offset: usize) -> Result<Vec<u8>, NszError> {
    if data.len() >= stream_offset + 8 && &data[stream_offset..stream_offset + 8] == b"NCZBLOCK" {
        return decode_ncz_block_stream(&data[stream_offset..]);
    }
    zstd::stream::decode_all(&data[stream_offset..]).map_err(NszError::from)
}

fn decode_ncz_block_stream(data: &[u8]) -> Result<Vec<u8>, NszError> {
    if data.len() < 24 {
        return Err(NszError::ContainerFormat {
            message: "NCZBLOCK header too short".to_string(),
        });
    }
    if &data[0..8] != b"NCZBLOCK" {
        return Err(NszError::ContainerFormat {
            message: "NCZBLOCK magic mismatch".to_string(),
        });
    }

    let block_size_exp = data[11];
    if !(14..=32).contains(&block_size_exp) {
        return Err(NszError::ContainerFormat {
            message: "NCZBLOCK block size exponent out of range".to_string(),
        });
    }
    let block_size = 1usize << block_size_exp;
    let number_of_blocks = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
    let decompressed_size = u64::from_le_bytes(data[16..24].try_into().unwrap()) as usize;

    let header_size = 24 + number_of_blocks * 4;
    if data.len() < header_size {
        return Err(NszError::ContainerFormat {
            message: "NCZBLOCK header truncated sizes list".to_string(),
        });
    }

    let mut compressed_sizes = Vec::with_capacity(number_of_blocks);
    let mut cursor = 24usize;
    for _ in 0..number_of_blocks {
        compressed_sizes
            .push(u32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap()) as usize);
        cursor += 4;
    }

    let mut stream_cursor = header_size;
    let mut out = Vec::with_capacity(decompressed_size);
    for compressed_size in compressed_sizes {
        if data.len() < stream_cursor + compressed_size {
            return Err(NszError::ContainerFormat {
                message: "NCZBLOCK stream truncated".to_string(),
            });
        }

        let remaining = decompressed_size.saturating_sub(out.len());
        let expected_block = remaining.min(block_size);
        let block_data = &data[stream_cursor..stream_cursor + compressed_size];
        stream_cursor += compressed_size;

        if compressed_size == expected_block {
            out.extend_from_slice(block_data);
            continue;
        }

        let decoded = zstd::stream::decode_all(block_data)?;
        if decoded.len() != expected_block {
            return Err(NszError::ContainerFormat {
                message: "NCZBLOCK decoded block size mismatch".to_string(),
            });
        }
        out.extend_from_slice(&decoded);
    }

    if out.len() != decompressed_size {
        return Err(NszError::ContainerFormat {
            message: "NCZBLOCK decompressed size mismatch".to_string(),
        });
    }
    Ok(out)
}

fn apply_aes_ctr(buf: &mut [u8], key: &[u8; 16], counter: &[u8; 16], offset: u128) {
    type AesCtr = ctr::Ctr128BE<Aes128>;
    let mut cipher = AesCtr::new(key.into(), counter.into());
    cipher.seek(offset);
    cipher.apply_keystream(buf);
}
