use crate::error::NszError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NspEntry {
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub string_table_offset: u32,
    pub reserved: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NspArchive {
    entries: Vec<NspEntry>,
    string_table_size: u32,
    data_start: u64,
}

impl NspArchive {
    pub fn from_bytes(data: &[u8]) -> Result<Self, NszError> {
        if data.len() < 16 {
            return Err(NszError::ContainerFormat {
                message: "PFS0 container too short".to_string(),
            });
        }
        if &data[0..4] != b"PFS0" {
            return Err(NszError::ContainerFormat {
                message: "PFS0 magic mismatch".to_string(),
            });
        }

        let file_count = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
        let string_table_size = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let entries_region_size =
            file_count
                .checked_mul(24)
                .ok_or_else(|| NszError::ContainerFormat {
                    message: "PFS0 entries size overflow".to_string(),
                })?;
        let header_size = 16usize
            .checked_add(entries_region_size)
            .and_then(|v| v.checked_add(string_table_size as usize))
            .ok_or_else(|| NszError::ContainerFormat {
                message: "PFS0 header size overflow".to_string(),
            })?;

        if data.len() < header_size {
            return Err(NszError::ContainerFormat {
                message: "PFS0 header truncated".to_string(),
            });
        }

        let mut raw_entries = Vec::with_capacity(file_count);
        let mut cursor = 16usize;
        for _ in 0..file_count {
            let offset = u64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap());
            let size = u64::from_le_bytes(data[cursor + 8..cursor + 16].try_into().unwrap());
            let string_table_offset =
                u32::from_le_bytes(data[cursor + 16..cursor + 20].try_into().unwrap());
            let reserved = u32::from_le_bytes(data[cursor + 20..cursor + 24].try_into().unwrap());
            raw_entries.push((offset, size, string_table_offset, reserved));
            cursor += 24;
        }

        let string_table = &data[cursor..cursor + string_table_size as usize];
        let max_data_end = raw_entries
            .iter()
            .map(|(offset, size, _, _)| offset.saturating_add(*size))
            .max()
            .unwrap_or(0);

        let data_start = (data.len() as u64)
            .checked_sub(max_data_end)
            .ok_or_else(|| NszError::ContainerFormat {
                message: "PFS0 data offsets exceed file size".to_string(),
            })?;

        if data_start < header_size as u64 {
            return Err(NszError::ContainerFormat {
                message: "PFS0 computed data start overlaps header".to_string(),
            });
        }

        let mut entries = Vec::with_capacity(file_count);
        for (offset, size, string_offset, reserved) in raw_entries {
            let string_offset = string_offset as usize;
            if string_offset >= string_table.len() {
                return Err(NszError::ContainerFormat {
                    message: "PFS0 string table offset out of bounds".to_string(),
                });
            }

            let name_end = string_table[string_offset..]
                .iter()
                .position(|b| *b == 0)
                .ok_or_else(|| NszError::ContainerFormat {
                    message: "PFS0 string table missing NUL terminator".to_string(),
                })?
                + string_offset;

            let name = std::str::from_utf8(&string_table[string_offset..name_end])
                .map_err(|_| NszError::ContainerFormat {
                    message: "PFS0 entry name is not valid UTF-8".to_string(),
                })?
                .to_string();

            let abs_start =
                data_start
                    .checked_add(offset)
                    .ok_or_else(|| NszError::ContainerFormat {
                        message: "PFS0 entry offset overflow".to_string(),
                    })?;
            let abs_end = abs_start
                .checked_add(size)
                .ok_or_else(|| NszError::ContainerFormat {
                    message: "PFS0 entry end overflow".to_string(),
                })?;

            if abs_end > data.len() as u64 {
                return Err(NszError::ContainerFormat {
                    message: format!("PFS0 entry {name} points outside file bounds"),
                });
            }

            entries.push(NspEntry {
                name,
                offset,
                size,
                string_table_offset: string_offset as u32,
                reserved,
            });
        }

        Ok(Self {
            entries,
            string_table_size,
            data_start,
        })
    }

    pub fn entries(&self) -> &[NspEntry] {
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

    pub fn entry_bytes<'a>(&self, data: &'a [u8], entry: &NspEntry) -> &'a [u8] {
        let start = (self.data_start + entry.offset) as usize;
        let end = start + entry.size as usize;
        &data[start..end]
    }
}

pub fn encode_pfs0(
    entries: &[(String, Vec<u8>)],
    first_file_offset: u64,
    base_string_table_size: u32,
) -> Result<Vec<u8>, NszError> {
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
            .checked_add(entries.len().checked_mul(24).ok_or_else(|| {
                NszError::ContainerFormat {
                    message: "PFS0 entry table size overflow".to_string(),
                }
            })?)
            .and_then(|v| v.checked_add(string_table_size))
            .ok_or_else(|| NszError::ContainerFormat {
                message: "PFS0 output header size overflow".to_string(),
            })?;

    if first_file_offset < header_size as u64 {
        return Err(NszError::ContainerFormat {
            message: "PFS0 first file offset is smaller than header size".to_string(),
        });
    }

    let mut header = Vec::with_capacity(header_size);
    header.extend_from_slice(b"PFS0");
    header.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    header.extend_from_slice(&(string_table_size as u32).to_le_bytes());
    header.extend_from_slice(&0u32.to_le_bytes());

    let mut abs_offset = first_file_offset;
    for ((_, data), string_offset) in entries.iter().zip(string_offsets.iter()) {
        let rel_offset = abs_offset.checked_sub(header_size as u64).ok_or_else(|| {
            NszError::ContainerFormat {
                message: "PFS0 relative offset underflow".to_string(),
            }
        })?;
        header.extend_from_slice(&rel_offset.to_le_bytes());
        header.extend_from_slice(&(data.len() as u64).to_le_bytes());
        header.extend_from_slice(&string_offset.to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes());
        abs_offset =
            abs_offset
                .checked_add(data.len() as u64)
                .ok_or_else(|| NszError::ContainerFormat {
                    message: "PFS0 absolute offset overflow".to_string(),
                })?;
    }

    header.extend_from_slice(&string_table);
    header.resize(header_size, 0);

    let mut out = header;
    out.resize(first_file_offset as usize, 0);
    for (_, data) in entries {
        out.extend_from_slice(data);
    }
    Ok(out)
}
