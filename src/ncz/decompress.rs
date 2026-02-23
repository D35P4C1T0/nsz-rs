use crate::error::NszError;

const UNCOMPRESSABLE_HEADER_SIZE: usize = 0x4000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NczSection {
    pub offset: u64,
    pub size: u64,
    pub crypto_type: u64,
    pub crypto_key: [u8; 16],
    pub crypto_counter: [u8; 16],
}

pub fn decompressed_nca_size_from_bytes(data: &[u8]) -> Result<u64, NszError> {
    let (sections, _) = parse_sections_with_end(data)?;
    let mut total = UNCOMPRESSABLE_HEADER_SIZE as u64;
    for section in sections {
        total = total
            .checked_add(section.size)
            .ok_or_else(|| NszError::ContainerFormat {
                message: "NCZ decompressed size overflow".to_string(),
            })?;
    }
    Ok(total)
}

pub fn decompress_ncz_to_vec(data: &[u8]) -> Result<Vec<u8>, NszError> {
    let (sections, stream_offset) = parse_sections_with_end(data)?;

    if sections
        .iter()
        .any(|s| s.crypto_type == 3 || s.crypto_type == 4)
    {
        return Err(NszError::UnsupportedFeature {
            feature: "NCZ AES-CTR re-encryption for crypto_type 3/4".to_string(),
        });
    }

    if data.len() < stream_offset {
        return Err(NszError::ContainerFormat {
            message: "NCZ stream offset outside input".to_string(),
        });
    }

    if data.len() >= stream_offset + 8 && &data[stream_offset..stream_offset + 8] == b"NCZBLOCK" {
        return Err(NszError::UnsupportedFeature {
            feature: "NCZBLOCK compressed stream".to_string(),
        });
    }

    let zstd_stream = &data[stream_offset..];
    let decompressed = zstd::stream::decode_all(zstd_stream)?;

    let mut output = Vec::with_capacity(decompressed_nca_size_from_bytes(data)? as usize);
    output.extend_from_slice(&data[..UNCOMPRESSABLE_HEADER_SIZE]);

    let mut read_cursor = 0usize;
    if let Some(first) = sections.first() {
        if first.offset > UNCOMPRESSABLE_HEADER_SIZE as u64 {
            let gap = (first.offset as usize) - UNCOMPRESSABLE_HEADER_SIZE;
            if decompressed.len() < gap {
                return Err(NszError::ContainerFormat {
                    message: "NCZ stream shorter than leading gap".to_string(),
                });
            }
            output.extend_from_slice(&decompressed[..gap]);
            read_cursor += gap;
        }
    }

    for section in &sections {
        let size = section.size as usize;
        let end = read_cursor + size;
        if decompressed.len() < end {
            return Err(NszError::ContainerFormat {
                message: "NCZ stream shorter than declared sections".to_string(),
            });
        }
        output.extend_from_slice(&decompressed[read_cursor..end]);
        read_cursor = end;
    }

    Ok(output)
}

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
