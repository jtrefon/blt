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
            let byte1 = parts[0].parse::<u8>().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse first byte value: {} in line '{}'", e, line),
                )
            })?;
            let byte2 = parts[1].parse::<u8>().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Failed to parse second byte value: {} in line '{}'",
                        e, line
                    ),
                )
            })?;
            merges.insert((byte1 as u16, byte2 as u16), vocab_size);
            vocab_size += 1;
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Invalid merge rule format in line: '{}'. Expected two numbers separated by space.", line)));
        }
    }
    Ok(merges)
}

// Other configuration loading functions can be added here later (e.g., for patchers).

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use std::collections::HashMap;

    fn create_merges_map(pairs: Vec<((u16, u16), u16)>) -> BpeMerges {
        pairs.into_iter().collect()
    }

    #[test]
    fn test_load_bpe_merges_valid() -> io::Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "97 98")?; // a b -> 256
        writeln!(file, "99 100")?; // c d -> 257
        writeln!(file, "# this is a comment")?;
        writeln!(file, "101 102")?; // e f -> 258
        file.flush()?;

        let merges = load_bpe_merges_from_path(file.path())?;
        let expected = create_merges_map(vec![
            ((97, 98), 256),
            ((99, 100), 257),
            ((101, 102), 258),
        ]);
        assert_eq!(merges, expected);
        Ok(())
    }

    #[test]
    fn test_load_bpe_merges_empty_file() -> io::Result<()> {
        let mut file = NamedTempFile::new()?; // Made file mutable
        // Intentionally left empty
        file.flush()?;

        let merges = load_bpe_merges_from_path(file.path())?;
        assert!(merges.is_empty());
        Ok(())
    }

    #[test]
    fn test_load_bpe_merges_only_comments_or_empty_lines() -> io::Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "# comment 1")?;
        writeln!(file, "")?; // Empty line
        writeln!(file, "# comment 2")?;
        file.flush()?;

        let merges = load_bpe_merges_from_path(file.path())?;
        assert!(merges.is_empty());
        Ok(())
    }

    #[test]
    fn test_load_bpe_merges_invalid_format_not_enough_parts() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "97").unwrap(); // Only one number
        file.flush().unwrap();

        let result = load_bpe_merges_from_path(file.path());
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind(), io::ErrorKind::InvalidData);
            assert!(e.to_string().contains("Invalid merge rule format"));
        }
    }

    #[test]
    fn test_load_bpe_merges_invalid_format_too_many_parts() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "97 98 99").unwrap(); // Three numbers
        file.flush().unwrap();

        let result = load_bpe_merges_from_path(file.path());
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind(), io::ErrorKind::InvalidData);
            assert!(e.to_string().contains("Invalid merge rule format"));
        }
    }

    #[test]
    fn test_load_bpe_merges_invalid_byte_value_nan() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "97 abc").unwrap(); // Second value not a number
        file.flush().unwrap();

        let result = load_bpe_merges_from_path(file.path());
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind(), io::ErrorKind::InvalidData);
            assert!(e.to_string().contains("Failed to parse second byte value"));
        }
    }

    #[test]
    fn test_load_bpe_merges_invalid_byte_value_overflow() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "256 98").unwrap(); // First value > 255 (u8 max)
        file.flush().unwrap();

        let result = load_bpe_merges_from_path(file.path());
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind(), io::ErrorKind::InvalidData);
            assert!(e.to_string().contains("Failed to parse first byte value"));
        }
    }

    #[test]
    fn test_load_bpe_merges_file_not_found() {
        let non_existent_path = Path::new("this_file_should_not_exist.txt");
        let result = load_bpe_merges_from_path(non_existent_path);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.kind(), io::ErrorKind::NotFound);
        }
    }
     #[test]
    fn test_vocab_size_increment() -> io::Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "1 2")?; // -> 256
        writeln!(file, "3 4")?; // -> 257
        writeln!(file, "1 2")?; // Duplicate, should not change vocab, but will overwrite.
                                // The spec doesn't explicitly state how to handle duplicate merge pairs.
                                // Current implementation overwrites with the latest vocab_id.
                                // For this test, we care that vocab_id increments correctly for new pairs.
        writeln!(file, "5 6")?; // -> 258 (assuming 1 2 was new, then 3 4, then 5 6)
                                // If 1 2 was overwritten, the id for 5 6 would still be 258 if it's the 3rd unique pair.
        file.flush()?;

        let merges = load_bpe_merges_from_path(file.path())?;
        let mut expected_merges = HashMap::new();
        // The vocab_size for "1 2" will be the one from its last appearance if duplicates map to new IDs.
        // However, the function uses a simple incrementing vocab_size for each valid line processed.
        // So, if "1 2" appears twice, it will be inserted twice with different vocab_ids if we didn't use a HashMap.
        // Since it's a HashMap, the last one wins.
        // Line 1: (1,2) -> 256
        // Line 2: (3,4) -> 257
        // Line 3: (1,2) -> 258 (overwrites (1,2)->256)
        // Line 4: (5,6) -> 259
        expected_merges.insert((1u16, 2u16), 258u16);
        expected_merges.insert((3u16, 4u16), 257u16);
        expected_merges.insert((5u16, 6u16), 259u16);


        assert_eq!(merges.len(), 3); // 3 unique pairs
        assert_eq!(merges, expected_merges);

        // Check that the values are what we expect from the incrementing vocab_size
        assert_eq!(merges.get(&(3,4)), Some(&257u16));
        assert_eq!(merges.get(&(1,2)), Some(&258u16)); // Last seen (1,2) gets vocab_id 258
        assert_eq!(merges.get(&(5,6)), Some(&259u16));

        Ok(())
    }
}
