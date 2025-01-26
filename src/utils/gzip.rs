use std::io::prelude::*;

use flate2::write::GzEncoder;
use flate2::Compression;

pub fn compress(content: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());

    let _ = encoder.write_all(content);

    let compressed = encoder.finish()?;

    Ok(compressed)
}
