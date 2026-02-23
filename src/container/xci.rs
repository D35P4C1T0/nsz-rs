use crate::container::hfs0::Hfs0Archive;
use crate::error::NszError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XciArchive {
    pub header_offset: u64,
    pub hfs0_offset: u64,
    pub hfs0_header_size: u64,
}

impl XciArchive {
    pub fn from_bytes(data: &[u8]) -> Result<Self, NszError> {
        let header_offset = if data.len() >= 0x104 && &data[0x100..0x104] == b"HEAD" {
            0u64
        } else {
            0x1000u64
        };
        let base = header_offset as usize;

        if data.len() < base + 0x148 {
            return Err(NszError::ContainerFormat {
                message: "XCI header truncated".to_string(),
            });
        }

        if &data[base + 0x100..base + 0x104] != b"HEAD" {
            return Err(NszError::ContainerFormat {
                message: "XCI header magic mismatch".to_string(),
            });
        }

        let hfs0_offset = u64::from_le_bytes(data[base + 0x138..base + 0x140].try_into().unwrap());
        let hfs0_header_size =
            u64::from_le_bytes(data[base + 0x140..base + 0x148].try_into().unwrap());
        let hfs0_abs_offset =
            header_offset
                .checked_add(hfs0_offset)
                .ok_or_else(|| NszError::ContainerFormat {
                    message: "XCI HFS0 offset overflow".to_string(),
                })?;

        if hfs0_abs_offset as usize >= data.len() {
            return Err(NszError::ContainerFormat {
                message: "XCI HFS0 offset outside file".to_string(),
            });
        }
        if hfs0_header_size > 0 && hfs0_abs_offset + hfs0_header_size > data.len() as u64 {
            return Err(NszError::ContainerFormat {
                message: "XCI HFS0 header range outside file".to_string(),
            });
        }

        Ok(Self {
            header_offset,
            hfs0_offset,
            hfs0_header_size,
        })
    }

    pub fn root_hfs0_bytes<'a>(&self, data: &'a [u8]) -> Result<&'a [u8], NszError> {
        let absolute = self
            .header_offset
            .checked_add(self.hfs0_offset)
            .ok_or_else(|| NszError::ContainerFormat {
                message: "XCI root HFS0 absolute offset overflow".to_string(),
            })? as usize;

        if absolute >= data.len() {
            return Err(NszError::ContainerFormat {
                message: "XCI root HFS0 offset outside file".to_string(),
            });
        }
        Ok(&data[absolute..])
    }

    pub fn root_hfs0_archive(&self, data: &[u8]) -> Result<Hfs0Archive, NszError> {
        Hfs0Archive::from_bytes(self.root_hfs0_bytes(data)?)
    }
}
