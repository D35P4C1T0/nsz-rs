use crate::error::NszError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHeader {
    pub version: u8,
    pub block_type: u8,
    pub unused: u8,
    pub block_size_exponent: u8,
    pub number_of_blocks: u32,
    pub decompressed_size: u64,
    pub compressed_block_sizes: Vec<u32>,
}

impl BlockHeader {
    pub fn from_bytes(data: &[u8]) -> Result<Self, NszError> {
        if data.len() < 24 {
            return Err(NszError::ContainerFormat {
                message: "NCZ block header too short".to_string(),
            });
        }
        if &data[0..8] != b"NCZBLOCK" {
            return Err(NszError::ContainerFormat {
                message: "NCZ block header magic mismatch".to_string(),
            });
        }

        let version = data[8];
        let block_type = data[9];
        let unused = data[10];
        let block_size_exponent = data[11];
        let number_of_blocks = u32::from_le_bytes(data[12..16].try_into().unwrap());
        let decompressed_size = u64::from_le_bytes(data[16..24].try_into().unwrap());
        let expected_len = 24 + (number_of_blocks as usize) * 4;

        if data.len() < expected_len {
            return Err(NszError::ContainerFormat {
                message: "NCZ block header truncated block size list".to_string(),
            });
        }

        let mut compressed_block_sizes = Vec::with_capacity(number_of_blocks as usize);
        let mut cursor = 24usize;
        for _ in 0..number_of_blocks {
            let size = u32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap());
            compressed_block_sizes.push(size);
            cursor += 4;
        }

        Ok(Self {
            version,
            block_type,
            unused,
            block_size_exponent,
            number_of_blocks,
            decompressed_size,
            compressed_block_sizes,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(24 + self.compressed_block_sizes.len() * 4);
        out.extend_from_slice(b"NCZBLOCK");
        out.push(self.version);
        out.push(self.block_type);
        out.push(self.unused);
        out.push(self.block_size_exponent);
        out.extend_from_slice(&(self.compressed_block_sizes.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.decompressed_size.to_le_bytes());
        for size in &self.compressed_block_sizes {
            out.extend_from_slice(&size.to_le_bytes());
        }
        out
    }
}
