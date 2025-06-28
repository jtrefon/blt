// blt_core/src/config_loader.rs
// For loading configurations like BPE merges from files.

use crate::BpeMerges; // Using the type alias from lib.rs
// use std::collections::HashMap; // Unused here as BpeMerges is from lib.rs
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub fn load_bpe_merges_from_path(path: &Path) -> io::Result<BpeMerges> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut merges = BpeMerges::new();
    let mut vocab_size = 256u16; // Start new tokens after byte values

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 {
            let byte1 = parts[0].parse::<u8>().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse first byte value: {} in line '{}'", e, line)))?;
            let byte2 = parts[1].parse::<u8>().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse second byte value: {} in line '{}'", e, line)))?;
            merges.insert((byte1 as u16, byte2 as u16), vocab_size);
            vocab_size += 1;
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid merge rule format in line: '{}'. Expected two numbers separated by space.", line)));
        }
    }
    Ok(merges)
}

// Other configuration loading functions can be added here later (e.g., for patchers).
