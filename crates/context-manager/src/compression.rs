//! Zstd compression wrapper

use anyhow::Result;
use std::io::{Read, Write};

pub struct Compressor {
    level: i32,
}

impl Compressor {
    pub fn new(level: i32) -> Self {
        Self { level: level.clamp(1, 22) }
    }

    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = zstd::Encoder::new(Vec::new(), self.level)?;
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = zstd::Decoder::new(data)?;
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let compressor = Compressor::new(10);
        let data = b"Hello, world! This is a test.";
        
        let compressed = compressor.compress(data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();
        
        assert_eq!(data, decompressed.as_slice());
    }
}
