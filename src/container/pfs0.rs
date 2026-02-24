use crate::error::NszError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pfs0FileEntry {
    pub offset: u64,
    pub size: u64,
    pub string_table_offset: u32,
    pub reserved: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pfs0Header {
    pub file_count: u32,
    pub string_table_size: u32,
    pub entries: Vec<Pfs0FileEntry>,
    pub string_table: Vec<u8>,
    pub padding: Vec<u8>,
}

impl Pfs0Header {
    /// Parses a raw PFS0 header region.
    pub fn from_bytes(data: &[u8]) -> Result<Self, NszError> {
        if data.len() < 16 {
            return Err(NszError::ContainerFormat {
                message: "PFS0 header too short".to_string(),
            });
        }

        if &data[0..4] != b"PFS0" {
            return Err(NszError::ContainerFormat {
                message: "PFS0 magic mismatch".to_string(),
            });
        }

        let file_count = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let string_table_size = u32::from_le_bytes(data[8..12].try_into().unwrap());

        let entries_size =
            (file_count as usize)
                .checked_mul(24)
                .ok_or_else(|| NszError::ContainerFormat {
                    message: "PFS0 entries size overflow".to_string(),
                })?;
        let header_without_padding = 16 + entries_size + string_table_size as usize;

        if data.len() < header_without_padding {
            return Err(NszError::ContainerFormat {
                message: "PFS0 truncated before string table end".to_string(),
            });
        }

        let mut entries = Vec::with_capacity(file_count as usize);
        let mut cursor = 16usize;
        for _ in 0..file_count {
            let offset = u64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap());
            let size = u64::from_le_bytes(data[cursor + 8..cursor + 16].try_into().unwrap());
            let string_table_offset =
                u32::from_le_bytes(data[cursor + 16..cursor + 20].try_into().unwrap());
            let reserved = u32::from_le_bytes(data[cursor + 20..cursor + 24].try_into().unwrap());
            entries.push(Pfs0FileEntry {
                offset,
                size,
                string_table_offset,
                reserved,
            });
            cursor += 24;
        }

        let string_table = data[cursor..cursor + string_table_size as usize].to_vec();
        cursor += string_table_size as usize;
        let padding = data[cursor..].to_vec();

        Ok(Self {
            file_count,
            string_table_size,
            entries,
            string_table,
            padding,
        })
    }

    /// Serializes the header back to its binary form.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(16 + self.entries.len() * 24 + self.string_table.len());
        out.extend_from_slice(b"PFS0");
        out.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());
        out.extend_from_slice(&(self.string_table.len() as u32).to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());

        for entry in &self.entries {
            out.extend_from_slice(&entry.offset.to_le_bytes());
            out.extend_from_slice(&entry.size.to_le_bytes());
            out.extend_from_slice(&entry.string_table_offset.to_le_bytes());
            out.extend_from_slice(&entry.reserved.to_le_bytes());
        }

        out.extend_from_slice(&self.string_table);
        out.extend_from_slice(&self.padding);
        out
    }
}
