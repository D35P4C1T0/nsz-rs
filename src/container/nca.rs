use std::collections::HashMap;

use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes128;
use ctr::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};

use crate::error::NszError;

const NCA_MEDIA_SIZE: u64 = 0x200;
const NCA_HEADER_SIZE: usize = 0xC00;
const NCA_SECTOR_SIZE: usize = 0x200;
const UNCOMPRESSABLE_HEADER_SIZE: u64 = 0x4000;
const BKTR_HEADER_SIZE: u64 = 0x4000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NcaCompressionMeta {
    pub content_type: u8,
    pub size: u64,
    pub packed: bool,
}

impl NcaCompressionMeta {
    pub fn is_compressible(&self) -> bool {
        matches!(self.content_type, 0x00 | 0x05) && self.packed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NcaEncryptionSection {
    pub offset: u64,
    pub size: u64,
    pub crypto_type: u64,
    pub crypto_key: [u8; 16],
    pub crypto_counter: [u8; 16],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NcaCompressionPlan {
    pub meta: NcaCompressionMeta,
    pub offset_first_section: u64,
    pub sections: Vec<NcaEncryptionSection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TicketRecord {
    pub rights_id: [u8; 16],
    pub encrypted_title_key: [u8; 16],
    pub master_key_revision: u8,
}

#[derive(Debug, Clone)]
pub struct NcaKeySet {
    pub header_key: [u8; 32],
    aes_kek_generation_source: [u8; 16],
    aes_key_generation_source: [u8; 16],
    titlekek_source: [u8; 16],
    key_area_key_application_source: [u8; 16],
    master_keys: HashMap<u8, [u8; 16]>,
    key_area_keys_application: HashMap<u8, [u8; 16]>,
}

impl NcaKeySet {
    pub fn from_keys_str(content: &str) -> Result<Self, NszError> {
        let mut raw = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let Some((name, value)) = trimmed.split_once('=') else {
                continue;
            };
            raw.insert(name.trim().to_string(), value.trim().to_string());
        }

        let header_key = parse_named_key::<32>(&raw, "header_key")?;
        let aes_kek_generation_source = parse_named_key::<16>(&raw, "aes_kek_generation_source")?;
        let aes_key_generation_source = parse_named_key::<16>(&raw, "aes_key_generation_source")?;
        let titlekek_source = parse_named_key::<16>(&raw, "titlekek_source")?;
        let key_area_key_application_source =
            parse_named_key::<16>(&raw, "key_area_key_application_source")?;

        let mut master_keys = HashMap::new();
        let mut key_area_keys_application = HashMap::new();
        for (name, value) in &raw {
            if let Some(suffix) = name.strip_prefix("master_key_") {
                if let Ok(index) = u8::from_str_radix(suffix, 16) {
                    let Some(key) = parse_hex_key::<16>(value) else {
                        return Err(NszError::ContainerFormat {
                            message: format!("invalid hex value for {name}"),
                        });
                    };
                    master_keys.insert(index, key);
                }
            }
            if let Some(suffix) = name.strip_prefix("key_area_key_application_") {
                if let Ok(index) = u8::from_str_radix(suffix, 16) {
                    let Some(key) = parse_hex_key::<16>(value) else {
                        return Err(NszError::ContainerFormat {
                            message: format!("invalid hex value for {name}"),
                        });
                    };
                    key_area_keys_application.insert(index, key);
                }
            }
        }

        if master_keys.is_empty() {
            return Err(NszError::ContainerFormat {
                message: "keys file does not contain any master_key_XX entries".to_string(),
            });
        }

        Ok(Self {
            header_key,
            aes_kek_generation_source,
            aes_key_generation_source,
            titlekek_source,
            key_area_key_application_source,
            master_keys,
            key_area_keys_application,
        })
    }

    fn master_key_index_from_header(header: &ParsedNcaHeader) -> u8 {
        let max_type = header.crypto_type.max(header.crypto_type2);
        max_type.saturating_sub(1)
    }

    fn master_key(&self, master_index: u8) -> Result<[u8; 16], NszError> {
        self.master_keys
            .get(&master_index)
            .copied()
            .ok_or_else(|| NszError::ContainerFormat {
                message: format!("missing master_key_{master_index:02x}"),
            })
    }

    fn title_kek(&self, master_index: u8) -> Result<[u8; 16], NszError> {
        let master_key = self.master_key(master_index)?;
        aes_ecb_decrypt_block(&master_key, &self.titlekek_source)
    }

    fn key_area_key_application(&self, master_index: u8) -> Result<[u8; 16], NszError> {
        if let Some(key) = self.key_area_keys_application.get(&master_index) {
            return Ok(*key);
        }
        let master_key = self.master_key(master_index)?;
        generate_kek(
            &self.key_area_key_application_source,
            &master_key,
            &self.aes_kek_generation_source,
            &self.aes_key_generation_source,
        )
    }

    fn resolve_title_key(
        &self,
        header: &ParsedNcaHeader,
        tickets: &HashMap<[u8; 16], TicketRecord>,
    ) -> Result<[u8; 16], NszError> {
        let master_index = Self::master_key_index_from_header(header);
        if header.rights_id != [0u8; 16] {
            let ticket =
                tickets
                    .get(&header.rights_id)
                    .ok_or_else(|| NszError::ContainerFormat {
                        message: format!(
                            "missing ticket for rights id {}",
                            hex_string(&header.rights_id)
                        ),
                    })?;
            let title_kek = self.title_kek(master_index)?;
            return aes_ecb_decrypt_block(&title_kek, &ticket.encrypted_title_key);
        }

        let key_area_key = self.key_area_key_application(master_index)?;
        let key_block = aes_ecb_decrypt(&key_area_key, &header.encrypted_key_block)?;
        let mut title_key = [0u8; 16];
        title_key.copy_from_slice(&key_block[0x20..0x30]);
        Ok(title_key)
    }
}

pub fn parse_ticket_record(data: &[u8]) -> Result<TicketRecord, NszError> {
    if data.len() < 4 {
        return Err(NszError::ContainerFormat {
            message: "ticket data too short".to_string(),
        });
    }

    let signature_type = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let signature_size = match signature_type {
        0x010000 | 0x010003 => 0x200usize,
        0x010001 | 0x010004 => 0x100usize,
        0x010002 | 0x010005 => 0x3Cusize,
        _ => {
            return Err(NszError::ContainerFormat {
                message: format!("unsupported ticket signature type: {signature_type:#x}"),
            });
        }
    };

    let signature_padding = 0x40usize - ((signature_size + 4) % 0x40);
    let base = 4usize + signature_size + signature_padding;
    if data.len() < base + 0x174 {
        return Err(NszError::ContainerFormat {
            message: "ticket data truncated".to_string(),
        });
    }

    let mut encrypted_title_key = [0u8; 16];
    encrypted_title_key.copy_from_slice(&data[base + 0x40..base + 0x50]);

    let mut rights_id = [0u8; 16];
    rights_id.copy_from_slice(&data[base + 0x160..base + 0x170]);

    let mut master_key_revision = data[base + 0x145];
    if master_key_revision == 0 {
        master_key_revision = data[base + 0x146];
    }

    Ok(TicketRecord {
        rights_id,
        encrypted_title_key,
        master_key_revision,
    })
}

pub fn analyze_for_compression(
    data: &[u8],
    header_key: &[u8; 32],
) -> Result<NcaCompressionMeta, NszError> {
    let header = parse_nca_header(data, header_key)?;
    Ok(header.meta())
}

pub fn build_compression_plan(
    data: &[u8],
    keys: &NcaKeySet,
    tickets: &HashMap<[u8; 16], TicketRecord>,
) -> Result<NcaCompressionPlan, NszError> {
    let header = parse_nca_header(data, &keys.header_key)?;
    let title_key = keys.resolve_title_key(&header, tickets)?;

    let mut sections = Vec::new();
    let offset_first_section = header
        .sections
        .first()
        .map(|s| s.offset)
        .unwrap_or(UNCOMPRESSABLE_HEADER_SIZE);
    for section in &header.sections {
        let normalized_crypto_type = match section.crypto_type {
            4 => 3u64,
            value => value as u64,
        };
        if section.bktr_subsection_size > 0 {
            let entries = parse_bktr_subsection_entries(data, section, &title_key)?;
            if entries.is_empty() {
                sections.push(NcaEncryptionSection {
                    offset: section.real_offset(),
                    size: section.size,
                    crypto_type: normalized_crypto_type,
                    crypto_key: title_key,
                    crypto_counter: section.crypto_counter,
                });
            } else {
                for entry in &entries {
                    sections.push(NcaEncryptionSection {
                        offset: section.real_offset().saturating_add(entry.virtual_offset),
                        size: entry.size,
                        crypto_type: normalized_crypto_type,
                        crypto_key: title_key,
                        crypto_counter: set_bktr_counter(section.crypto_counter, entry.ctr),
                    });
                }
                if let Some(last) = sections.last() {
                    let next_offset = last.offset.saturating_add(last.size);
                    let section_end = section.offset.saturating_add(section.size);
                    sections.push(NcaEncryptionSection {
                        offset: next_offset,
                        size: section_end.saturating_sub(next_offset),
                        crypto_type: normalized_crypto_type,
                        crypto_key: title_key,
                        crypto_counter: section.crypto_counter,
                    });
                }
            }
            continue;
        }

        sections.push(NcaEncryptionSection {
            offset: section.real_offset(),
            size: section.size,
            crypto_type: normalized_crypto_type,
            crypto_key: title_key,
            crypto_counter: section.crypto_counter,
        });
    }

    Ok(NcaCompressionPlan {
        meta: header.meta(),
        offset_first_section,
        sections,
    })
}

#[derive(Debug, Clone, Copy)]
struct BktrSubsectionEntry {
    virtual_offset: u64,
    size: u64,
    ctr: u32,
}

#[derive(Debug, Clone)]
struct ParsedSection {
    offset: u64,
    size: u64,
    crypto_type: u8,
    crypto_counter: [u8; 16],
    section_start: u64,
    bktr_subsection_offset: u64,
    bktr_subsection_size: u64,
}

impl ParsedSection {
    fn real_offset(&self) -> u64 {
        self.offset.saturating_sub(self.section_start)
    }
}

#[derive(Debug, Clone)]
struct ParsedNcaHeader {
    content_type: u8,
    size: u64,
    crypto_type: u8,
    crypto_type2: u8,
    rights_id: [u8; 16],
    encrypted_key_block: [u8; 64],
    sections: Vec<ParsedSection>,
}

impl ParsedNcaHeader {
    fn meta(&self) -> NcaCompressionMeta {
        let packed = if self.sections.is_empty() {
            true
        } else {
            let mut next = self.sections[0].offset;
            let mut contiguous = true;
            for section in &self.sections {
                if section.offset != next {
                    contiguous = false;
                    break;
                }
                next = section.offset.saturating_add(section.size);
            }
            contiguous && next == self.size
        };
        NcaCompressionMeta {
            content_type: self.content_type,
            size: self.size,
            packed,
        }
    }
}

fn parse_nca_header(data: &[u8], header_key: &[u8; 32]) -> Result<ParsedNcaHeader, NszError> {
    if data.len() < NCA_HEADER_SIZE {
        return Err(NszError::ContainerFormat {
            message: "NCA data too short for header analysis".to_string(),
        });
    }

    let mut header = data[..NCA_HEADER_SIZE].to_vec();
    decrypt_nca_header_xts(&mut header, header_key)?;

    if &header[0x200..0x204] != b"NCA2" && &header[0x200..0x204] != b"NCA3" {
        return Err(NszError::ContainerFormat {
            message: "NCA header magic mismatch after XTS decryption".to_string(),
        });
    }

    let content_type = header[0x205];
    let crypto_type = header[0x206];
    let size = u64::from_le_bytes(header[0x208..0x210].try_into().unwrap());
    let crypto_type2 = header[0x220];

    let mut rights_id = [0u8; 16];
    rights_id.copy_from_slice(&header[0x230..0x240]);

    let mut encrypted_key_block = [0u8; 64];
    encrypted_key_block.copy_from_slice(&header[0x300..0x340]);

    let mut sections = Vec::new();
    for section_index in 0..4 {
        let table_cursor = 0x240usize + section_index * 0x10;
        let media_offset =
            u32::from_le_bytes(header[table_cursor..table_cursor + 4].try_into().unwrap()) as u64;
        let media_end_offset = u32::from_le_bytes(
            header[table_cursor + 4..table_cursor + 8]
                .try_into()
                .unwrap(),
        ) as u64;
        let offset = media_offset.saturating_mul(NCA_MEDIA_SIZE);
        let end = media_end_offset.saturating_mul(NCA_MEDIA_SIZE);
        if end <= offset || end > size {
            continue;
        }

        let section_header_start = 0x400usize + section_index * 0x200;
        let section_header = &header[section_header_start..section_header_start + 0x200];
        let crypto_type_section = section_header[0x4];

        let mut crypto_counter = [0u8; 16];
        crypto_counter[8..16].copy_from_slice(&section_header[0x140..0x148]);
        crypto_counter.reverse();

        let bktr_subsection_offset =
            u64::from_le_bytes(section_header[0x120..0x128].try_into().unwrap());
        let bktr_subsection_size =
            u64::from_le_bytes(section_header[0x128..0x130].try_into().unwrap());

        sections.push(ParsedSection {
            offset,
            size: end.saturating_sub(offset),
            crypto_type: crypto_type_section,
            crypto_counter,
            section_start: 0,
            bktr_subsection_offset,
            bktr_subsection_size,
        });
    }

    sections.sort_by_key(|section| section.offset);
    Ok(ParsedNcaHeader {
        content_type,
        size,
        crypto_type,
        crypto_type2,
        rights_id,
        encrypted_key_block,
        sections,
    })
}

fn parse_bktr_subsection_entries(
    data: &[u8],
    section: &ParsedSection,
    title_key: &[u8; 16],
) -> Result<Vec<BktrSubsectionEntry>, NszError> {
    if section
        .bktr_subsection_offset
        .saturating_add(BKTR_HEADER_SIZE)
        > section.size
    {
        return Err(NszError::ContainerFormat {
            message: "BKTR subsection header outside section bounds".to_string(),
        });
    }

    let header = read_section_range(
        data,
        section,
        section.bktr_subsection_offset,
        BKTR_HEADER_SIZE as usize,
        title_key,
    )?;
    if header.len() < BKTR_HEADER_SIZE as usize {
        return Err(NszError::ContainerFormat {
            message: "BKTR subsection header truncated".to_string(),
        });
    }

    let bucket_count = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
    let mut cursor = section
        .bktr_subsection_offset
        .saturating_add(BKTR_HEADER_SIZE);
    let mut out: Vec<BktrSubsectionEntry> = Vec::new();

    for _ in 0..bucket_count {
        let bucket_header = read_section_range(data, section, cursor, 0x10, title_key)?;
        let entry_count = u32::from_le_bytes(bucket_header[4..8].try_into().unwrap()) as usize;
        let end_offset = u64::from_le_bytes(bucket_header[8..16].try_into().unwrap());
        cursor = cursor.saturating_add(0x10);

        let entries_bytes = read_section_range(
            data,
            section,
            cursor,
            entry_count.saturating_mul(0x10),
            title_key,
        )?;
        cursor = cursor.saturating_add((entry_count as u64).saturating_mul(0x10));

        let start_index = out.len();
        for entry_index in 0..entry_count {
            let base = entry_index * 0x10;
            let virtual_offset =
                u64::from_le_bytes(entries_bytes[base..base + 8].try_into().unwrap());
            let ctr = u32::from_le_bytes(entries_bytes[base + 12..base + 16].try_into().unwrap());
            if entry_index > 0 {
                let previous = out.last_mut().ok_or_else(|| NszError::ContainerFormat {
                    message: "BKTR subsection state underflow".to_string(),
                })?;
                previous.size = virtual_offset.saturating_sub(previous.virtual_offset);
            }
            out.push(BktrSubsectionEntry {
                virtual_offset,
                size: 0,
                ctr,
            });
        }
        if out.len() > start_index {
            let last = out.last_mut().ok_or_else(|| NszError::ContainerFormat {
                message: "BKTR subsection missing entries".to_string(),
            })?;
            last.size = end_offset.saturating_sub(last.virtual_offset);
        }
    }

    Ok(out)
}

fn read_section_range(
    data: &[u8],
    section: &ParsedSection,
    relative_offset: u64,
    size: usize,
    title_key: &[u8; 16],
) -> Result<Vec<u8>, NszError> {
    let end_relative = relative_offset.saturating_add(size as u64);
    if end_relative > section.size {
        return Err(NszError::ContainerFormat {
            message: "section read outside bounds".to_string(),
        });
    }

    let absolute_offset = section.offset.saturating_add(relative_offset);
    let end_absolute = absolute_offset.saturating_add(size as u64);
    if end_absolute > data.len() as u64 {
        return Err(NszError::ContainerFormat {
            message: "section read exceeds NCA file size".to_string(),
        });
    }

    let mut bytes = data[absolute_offset as usize..end_absolute as usize].to_vec();
    if matches!(section.crypto_type, 3 | 4) {
        apply_aes_ctr(
            &mut bytes,
            title_key,
            &section.crypto_counter,
            absolute_offset as u128,
        );
    }
    Ok(bytes)
}

fn parse_named_key<const N: usize>(
    values: &HashMap<String, String>,
    name: &str,
) -> Result<[u8; N], NszError> {
    let value = values.get(name).ok_or_else(|| NszError::ContainerFormat {
        message: format!("keys file missing {name}"),
    })?;
    parse_hex_key::<N>(value).ok_or_else(|| NszError::ContainerFormat {
        message: format!("invalid hex value for {name}"),
    })
}

fn parse_hex_key<const N: usize>(value: &str) -> Option<[u8; N]> {
    if value.len() != N * 2 {
        return None;
    }
    let mut out = [0u8; N];
    for (index, slot) in out.iter_mut().enumerate() {
        let hi = hex_nibble(value.as_bytes()[index * 2])?;
        let lo = hex_nibble(value.as_bytes()[index * 2 + 1])?;
        *slot = (hi << 4) | lo;
    }
    Some(out)
}

fn generate_kek(
    src: &[u8; 16],
    master_key: &[u8; 16],
    kek_seed: &[u8; 16],
    key_seed: &[u8; 16],
) -> Result<[u8; 16], NszError> {
    let kek = aes_ecb_decrypt_block(master_key, kek_seed)?;
    let src_kek = aes_ecb_decrypt_block(&kek, src)?;
    aes_ecb_decrypt_block(&src_kek, key_seed)
}

fn aes_ecb_decrypt_block(key: &[u8; 16], block: &[u8; 16]) -> Result<[u8; 16], NszError> {
    let cipher = Aes128::new_from_slice(key).map_err(|_| NszError::ContainerFormat {
        message: "invalid AES-128 key".to_string(),
    })?;
    let mut out = *block;
    cipher.decrypt_block(GenericArray::from_mut_slice(&mut out));
    Ok(out)
}

fn aes_ecb_decrypt(key: &[u8; 16], data: &[u8]) -> Result<Vec<u8>, NszError> {
    if !data.len().is_multiple_of(16) {
        return Err(NszError::ContainerFormat {
            message: "AES-ECB data is not 16-byte aligned".to_string(),
        });
    }
    let cipher = Aes128::new_from_slice(key).map_err(|_| NszError::ContainerFormat {
        message: "invalid AES-128 key".to_string(),
    })?;
    let mut out = data.to_vec();
    for block in out.chunks_exact_mut(16) {
        cipher.decrypt_block(GenericArray::from_mut_slice(block));
    }
    Ok(out)
}

fn apply_aes_ctr(buf: &mut [u8], key: &[u8; 16], counter: &[u8; 16], offset: u128) {
    type AesCtr = ctr::Ctr128BE<Aes128>;
    let mut cipher = AesCtr::new(key.into(), counter.into());
    cipher.seek(offset);
    cipher.apply_keystream(buf);
}

fn set_bktr_counter(base_counter: [u8; 16], ctr_value: u32) -> [u8; 16] {
    let mut counter = base_counter;
    for index in 0..8 {
        counter[15 - index] = 0;
    }
    let mut value = ctr_value;
    for index in 0..4 {
        counter[7 - index] = (value & 0xFF) as u8;
        value >>= 8;
    }
    counter
}

fn hex_string(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 2);
    for byte in data {
        out.push(hex_digit(byte >> 4));
        out.push(hex_digit(byte & 0x0F));
    }
    out
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'a' + (value - 10)) as char,
    }
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn decrypt_nca_header_xts(data: &mut [u8], header_key: &[u8; 32]) -> Result<(), NszError> {
    if !data.len().is_multiple_of(NCA_SECTOR_SIZE) {
        return Err(NszError::ContainerFormat {
            message: "NCA header size is not aligned to XTS sector size".to_string(),
        });
    }

    let cipher_data =
        Aes128::new_from_slice(&header_key[..16]).map_err(|_| NszError::ContainerFormat {
            message: "invalid NCA XTS data key length".to_string(),
        })?;
    let cipher_tweak =
        Aes128::new_from_slice(&header_key[16..]).map_err(|_| NszError::ContainerFormat {
            message: "invalid NCA XTS tweak key length".to_string(),
        })?;

    for (sector_index, sector) in data.chunks_exact_mut(NCA_SECTOR_SIZE).enumerate() {
        let mut tweak = [0u8; 16];
        tweak[8..16].copy_from_slice(&(sector_index as u64).to_be_bytes());
        let mut tweak_block = tweak;
        cipher_tweak.encrypt_block(GenericArray::from_mut_slice(&mut tweak_block));

        for block in sector.chunks_exact_mut(16) {
            for (index, value) in block.iter_mut().enumerate() {
                *value ^= tweak_block[index];
            }
            cipher_data.decrypt_block(GenericArray::from_mut_slice(block));
            for (index, value) in block.iter_mut().enumerate() {
                *value ^= tweak_block[index];
            }
            xts_mul_alpha_le(&mut tweak_block);
        }
    }

    Ok(())
}

fn xts_mul_alpha_le(tweak: &mut [u8; 16]) {
    let mut carry = 0u8;
    for byte in tweak.iter_mut() {
        let new_carry = *byte >> 7;
        *byte = (*byte << 1) | carry;
        carry = new_carry;
    }
    if carry != 0 {
        tweak[0] ^= 0x87;
    }
}
