use crate::error::NszError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hfs0Entry {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub string_table_offset: u32,
    pub hashed_region_size: u32,
    pub reserved: [u8; 8],
    pub hash: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hfs0Archive {
    entries: Vec<Hfs0Entry>,
    string_table_size: u32,
    data_start: u64,
}

impl Hfs0Archive {
    pub fn from_bytes(data: &[u8]) -> Result<Self, NszError> {
        if data.len() < 16 {
            return Err(NszError::ContainerFormat {
                message: "HFS0 container too short".to_string(),
            });
        }
        if &data[0..4] != b"HFS0" {
            return Err(NszError::ContainerFormat {
                message: "HFS0 magic mismatch".to_string(),
            });
        }

        let file_count = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
        let string_table_size = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
        let entries_region_size =
            file_count
                .checked_mul(0x40)
                .ok_or_else(|| NszError::ContainerFormat {
                    message: "HFS0 entries size overflow".to_string(),
                })?;
        let header_size = 16usize
            .checked_add(entries_region_size)
            .and_then(|v| v.checked_add(string_table_size))
            .ok_or_else(|| NszError::ContainerFormat {
                message: "HFS0 header size overflow".to_string(),
            })?;

        if data.len() < header_size {
            return Err(NszError::ContainerFormat {
                message: "HFS0 header truncated".to_string(),
            });
        }

        let mut raw_entries = Vec::with_capacity(file_count);
        let mut cursor = 16usize;
        for _ in 0..file_count {
            let offset = u64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap());
            let size = u64::from_le_bytes(data[cursor + 8..cursor + 16].try_into().unwrap());
            let string_table_offset =
                u32::from_le_bytes(data[cursor + 16..cursor + 20].try_into().unwrap());
            let hashed_region_size =
                u32::from_le_bytes(data[cursor + 20..cursor + 24].try_into().unwrap());
            let reserved: [u8; 8] = data[cursor + 24..cursor + 32].try_into().unwrap();
            let hash: [u8; 32] = data[cursor + 32..cursor + 64].try_into().unwrap();
            raw_entries.push((
                offset,
                size,
                string_table_offset,
                hashed_region_size,
                reserved,
                hash,
            ));
            cursor += 0x40;
        }

        let string_table = &data[cursor..cursor + string_table_size];
        let data_start = header_size as u64;

        let mut entries = Vec::with_capacity(file_count);
        for (offset, size, string_offset, hashed_region_size, reserved, hash) in raw_entries {
            let string_offset = string_offset as usize;
            if string_offset >= string_table.len() {
                return Err(NszError::ContainerFormat {
                    message: "HFS0 string table offset out of bounds".to_string(),
                });
            }
            let name_end = string_table[string_offset..]
                .iter()
                .position(|b| *b == 0)
                .ok_or_else(|| NszError::ContainerFormat {
                    message: "HFS0 string table missing NUL terminator".to_string(),
                })?
                + string_offset;
            let name = std::str::from_utf8(&string_table[string_offset..name_end])
                .map_err(|_| NszError::ContainerFormat {
                    message: "HFS0 entry name is not valid UTF-8".to_string(),
                })?
                .to_string();

            let abs_start =
                data_start
                    .checked_add(offset)
                    .ok_or_else(|| NszError::ContainerFormat {
                        message: "HFS0 entry offset overflow".to_string(),
                    })?;
            let abs_end = abs_start
                .checked_add(size)
                .ok_or_else(|| NszError::ContainerFormat {
                    message: "HFS0 entry end overflow".to_string(),
                })?;
            if abs_end > data.len() as u64 {
                return Err(NszError::ContainerFormat {
                    message: format!("HFS0 entry {name} points outside file bounds"),
                });
            }

            entries.push(Hfs0Entry {
                name,
                offset,
                size,
                string_table_offset: string_offset as u32,
                hashed_region_size,
                reserved,
                hash,
            });
        }

        Ok(Self {
            entries,
            string_table_size: string_table_size as u32,
            data_start,
        })
    }

    pub fn entries(&self) -> &[Hfs0Entry] {
        &self.entries
    }

    pub fn string_table_size(&self) -> u32 {
        self.string_table_size
    }

    pub fn first_file_offset(&self) -> u64 {
        self.entries
            .iter()
            .map(|entry| self.data_start + entry.offset)
            .min()
            .unwrap_or(self.data_start)
    }

    pub fn entry_bytes<'a>(&self, data: &'a [u8], entry: &Hfs0Entry) -> &'a [u8] {
        let start = (self.data_start + entry.offset) as usize;
        let end = start + entry.size as usize;
        &data[start..end]
    }
}

pub fn encode_hfs0<B: AsRef<[u8]>>(
    entries: &[(String, B)],
    first_file_offset: u64,
    base_string_table_size: u32,
) -> Result<Vec<u8>, NszError> {
    let payloads: Vec<&[u8]> = entries.iter().map(|(_, data)| data.as_ref()).collect();
    let mut string_table = Vec::new();
    let mut string_offsets = Vec::with_capacity(entries.len());
    for (name, _) in entries {
        string_offsets.push(string_table.len() as u32);
        string_table.extend_from_slice(name.as_bytes());
        string_table.push(0);
    }

    let string_table_size = (base_string_table_size as usize).max(string_table.len());
    let header_size =
        16usize
            .checked_add(entries.len().checked_mul(0x40).ok_or_else(|| {
                NszError::ContainerFormat {
                    message: "HFS0 entry table size overflow".to_string(),
                }
            })?)
            .and_then(|v| v.checked_add(string_table_size))
            .ok_or_else(|| NszError::ContainerFormat {
                message: "HFS0 output header size overflow".to_string(),
            })?;

    if first_file_offset < header_size as u64 {
        return Err(NszError::ContainerFormat {
            message: "HFS0 first file offset is smaller than header size".to_string(),
        });
    }

    let mut header = Vec::with_capacity(header_size);
    header.extend_from_slice(b"HFS0");
    header.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    header.extend_from_slice(&(string_table_size as u32).to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());

    let mut abs_offset = first_file_offset;
    for (payload, string_offset) in payloads.iter().zip(string_offsets.iter()) {
        let rel_offset = abs_offset.checked_sub(header_size as u64).ok_or_else(|| {
            NszError::ContainerFormat {
                message: "HFS0 relative offset underflow".to_string(),
            }
        })?;

        header.extend_from_slice(&rel_offset.to_le_bytes());
        header.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        header.extend_from_slice(&string_offset.to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes());
        header.extend_from_slice(&0u64.to_le_bytes());
        header.extend_from_slice(&[0u8; 32]);

        abs_offset = abs_offset
            .checked_add(payload.len() as u64)
            .ok_or_else(|| NszError::ContainerFormat {
                message: "HFS0 absolute offset overflow".to_string(),
            })?;
    }

    header.extend_from_slice(&string_table);
    header.resize(header_size, 0);

    let first_file_offset = usize::try_from(first_file_offset).map_err(|_| {
        NszError::ContainerFormat {
            message: "HFS0 first file offset does not fit usize".to_string(),
        }
    })?;
    let payload_size = payloads.iter().try_fold(0usize, |acc, payload| {
        acc.checked_add(payload.len()).ok_or_else(|| NszError::ContainerFormat {
            message: "HFS0 payload size overflow".to_string(),
        })
    })?;
    let total_size = first_file_offset
        .checked_add(payload_size)
        .ok_or_else(|| NszError::ContainerFormat {
            message: "HFS0 output size overflow".to_string(),
        })?;

    let mut out = Vec::with_capacity(total_size);
    out.extend_from_slice(&header);
    out.resize(first_file_offset, 0);
    for payload in payloads {
        out.extend_from_slice(payload);
    }
    Ok(out)
}
