// blt_core/src/tokenizer.rs
// Contains tokenization strategies (BPE, patching, etc.).

use crate::BpeMerges; // Using the type alias from lib.rs
use std::io;
use std::sync::Arc;

// This is the core BPE processing logic, adapted from the previous main.rs
pub async fn process_chunk_bpe(
    chunk_data: Vec<u8>,
    bpe_merges: Arc<BpeMerges>, // Assuming BPE merges are always provided if this fn is called
) -> io::Result<Vec<u8>> {
    // Initial sequence of u16 tokens (bytes promoted to u16)
    let mut tokens: Vec<u16> = chunk_data.into_iter().map(|b| b as u16).collect();

    loop {
        let mut new_tokens = Vec::with_capacity(tokens.len());
        let mut i = 0;
        let mut merged_this_pass = false;

        while i < tokens.len() {
            if i + 1 < tokens.len() {
                if let Some(&new_token) = bpe_merges.get(&(tokens[i], tokens[i + 1])) {
                    new_tokens.push(new_token);
                    i += 2;
                    merged_this_pass = true;
                } else {
                    new_tokens.push(tokens[i]);
                    i += 1;
                }
            } else {
                new_tokens.push(tokens[i]);
                i += 1;
            }
        }
        tokens = new_tokens;
        if !merged_this_pass {
            break;
        }
    }

    // Convert u16 tokens back to Vec<u8> for output (big-endian).
    let mut output_bytes = Vec::with_capacity(tokens.len() * 2);
    for token in tokens {
        output_bytes.extend_from_slice(&token.to_be_bytes());
    }
    Ok(output_bytes)
}

// General chunk processing function.
// It will decide whether to apply BPE or other strategies based on config.
// For now, it assumes if bpe_data is in CoreConfig, BPE should be applied.
pub async fn process_chunk(
    chunk_data: Vec<u8>,
    bpe_data_opt: Option<Arc<BpeMerges>>, // Passed from CoreConfig
                                          // Potentially other strategy configs later
) -> io::Result<Vec<u8>> {
    if let Some(bpe_merges) = bpe_data_opt {
        // If BPE merges are available, use BPE processing
        process_chunk_bpe(chunk_data, bpe_merges).await
    } else {
        // No BPE (or other strategies yet), return original chunk bytes
        Ok(chunk_data)
    }
}

// This module could later include:
// - Traits for different tokenization strategies.
// - Implementations for other strategies (e.g., patch-based).
// - Logic to select and apply strategies based on configuration.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BpeMerges;
    use std::sync::Arc;
    // Removed: use std::collections::HashMap; (It was unused)

    fn u8_slice_to_u16_vec(slice: &[u8]) -> Vec<u16> {
        slice.iter().map(|&b| b as u16).collect()
    }

    fn u16_vec_to_byte_vec(tokens: &[u16]) -> Vec<u8> {
        tokens.iter().flat_map(|&t| t.to_be_bytes()).collect()
    }

    fn create_bpe_arc(pairs: Vec<((u16, u16), u16)>) -> Arc<BpeMerges> {
        Arc::new(pairs.into_iter().collect())
    }

    #[tokio::test]
    async fn test_process_chunk_bpe_no_merges() -> io::Result<()> {
        let bpe_data = create_bpe_arc(vec![]);
        let chunk = b"abcdef".to_vec();
        let expected_tokens = u8_slice_to_u16_vec(b"abcdef");

        let result = process_chunk_bpe(chunk.clone(), bpe_data).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_process_chunk_bpe_simple_merge() -> io::Result<()> {
        // 'a' (97) 'b' (98) -> 256
        let bpe_data = create_bpe_arc(vec![((97, 98), 256)]);
        let chunk = b"abcab".to_vec();
        // Expected: 256, 'c' (99), 256
        let expected_tokens = vec![256, 99, 256];

        let result = process_chunk_bpe(chunk, bpe_data).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_process_chunk_bpe_multiple_merges() -> io::Result<()> {
        // 'a' (97) 'b' (98) -> 256
        // 'c' (99) 'd' (100) -> 257
        let bpe_data = create_bpe_arc(vec![((97, 98), 256), ((99, 100), 257)]);
        let chunk = b"abcdab".to_vec();
        // Expected: 256, 257, 256
        let expected_tokens = vec![256, 257, 256];

        let result = process_chunk_bpe(chunk, bpe_data).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_process_chunk_bpe_iterative_merging() -> io::Result<()> {
        // 'a' (97) 'b' (98) -> 256
        // 256 'c' (99) -> 257
        let bpe_data = create_bpe_arc(vec![((97, 98), 256), ((256, 99), 257)]);
        let chunk = b"abcde".to_vec();
        // Pass 1: "ab"cde -> 256 cde
        // Pass 2: 256c de -> 257 de
        let expected_tokens = vec![257, 100, 101]; // 257 ('d'), 'e'

        let result = process_chunk_bpe(chunk, bpe_data).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_process_chunk_bpe_no_possible_merges() -> io::Result<()> {
        let bpe_data = create_bpe_arc(vec![((97, 98), 256)]); // 'a' 'b' -> 256
        let chunk = b"xyz123".to_vec();
        let expected_tokens = u8_slice_to_u16_vec(b"xyz123");

        let result = process_chunk_bpe(chunk, bpe_data).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_process_chunk_bpe_empty_input() -> io::Result<()> {
        let bpe_data = create_bpe_arc(vec![((97, 98), 256)]);
        let chunk = b"".to_vec();
        let expected_tokens: Vec<u16> = vec![];

        let result = process_chunk_bpe(chunk, bpe_data).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_process_chunk_bpe_single_byte_input_cannot_merge() -> io::Result<()> {
        let bpe_data = create_bpe_arc(vec![((97, 98), 256)]);
        let chunk = b"a".to_vec();
        let expected_tokens = vec![97u16];

        let result = process_chunk_bpe(chunk, bpe_data).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_process_chunk_with_bpe_data() -> io::Result<()> {
        // 'a' (97) 'b' (98) -> 256
        let bpe_data_map: BpeMerges = vec![((97u16, 98u16), 256u16)].into_iter().collect();
        let bpe_data_arc = Some(Arc::new(bpe_data_map));
        let chunk = b"ab c".to_vec();
        // Expected: 256, ' ' (32), 'c' (99)
        let expected_tokens = vec![256, 32, 99];

        let result = process_chunk(chunk.clone(), bpe_data_arc).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_process_chunk_without_bpe_data() -> io::Result<()> {
        let chunk = b"ab c".to_vec();
        // Expected: 'a', 'b', ' ', 'c' (original bytes)
        let expected_bytes = chunk.clone(); // Original chunk data as is

        let result = process_chunk(chunk, None).await?; // Pass None for bpe_data_opt
        assert_eq!(result, expected_bytes);
        Ok(())
    }

     #[tokio::test]
    async fn test_process_chunk_bpe_merge_produces_byte_value() -> io::Result<()> {
        // Test a merge that results in a token ID that is itself a valid byte value (e.g. < 256)
        // 'x' (120) 'y' (121) -> 'Z' (90)
        let bpe_data = create_bpe_arc(vec![((120, 121), 90)]);
        let chunk = b"axyza".to_vec();
        // Expected: 'a' (97), 'Z' (90), 'z' (122), 'a' (97)
        let expected_tokens = vec![97, 90, 122, 97]; // Corrected expectation

        let result = process_chunk_bpe(chunk, bpe_data).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }
}
