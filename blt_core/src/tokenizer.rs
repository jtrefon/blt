//! Defines the core traits and implementations for tokenization strategies.
//!
//! This module provides the `TokenizationStrategy` trait, which allows for different
//! tokenization algorithms to be used interchangeably within the processing pipeline.
//! It includes a `BpeStrategy` for Byte-Pair Encoding and a `PassthroughStrategy`
//! as a default no-op.

use crate::BpeMerges;
use async_trait;
use std::io;
use std::sync::Arc;
use tracing::{debug, instrument};

// --- Tokenization Strategy Trait ---

/// A trait that defines the interface for a tokenization algorithm.
///
/// A strategy is responsible for processing a single chunk of bytes (`&[u8]`) and returning
/// the tokenized result. This allows the core pipeline to remain agnostic about the specific
/// tokenization logic being applied.
#[async_trait::async_trait]
pub trait TokenizationStrategy: Send + Sync {
    /// Processes a chunk of data asynchronously.
    ///
    /// # Arguments
    /// * `chunk_data` - A slice of bytes representing the data chunk to be processed.
    ///
    /// # Returns
    /// A `Result` containing the processed `Vec<u8>` on success, or an `io::Error` on failure.
    async fn process_chunk(&self, chunk_data: &[u8]) -> io::Result<Vec<u8>>;
}

// --- BPE Strategy Implementation ---

/// A tokenization strategy that applies Byte-Pair Encoding (BPE).
///
/// This strategy iteratively merges the most frequent pairs of bytes into new, single tokens
/// based on a provided `merges` map.
pub struct BpeStrategy {
    bpe_merges: Arc<BpeMerges>,
}

impl BpeStrategy {
    /// Creates a new `BpeStrategy` with the given BPE merges.
    ///
    /// # Arguments
    /// * `bpe_merges` - An `Arc`-wrapped map of byte pairs to their resulting merged token.
    pub fn new(bpe_merges: Arc<BpeMerges>) -> Self {
        Self { bpe_merges }
    }
}

#[async_trait::async_trait]
impl TokenizationStrategy for BpeStrategy {
    #[instrument(skip(self, chunk_data), name = "bpe_strategy_process")]
    async fn process_chunk(&self, chunk_data: &[u8]) -> io::Result<Vec<u8>> {
        if chunk_data.is_empty() {
            return Ok(Vec::new());
        }

        let mut tokens: Vec<u16> = chunk_data.iter().map(|&b| b as u16).collect();

        loop {
            let mut merges_found = false;
            let mut new_tokens = Vec::with_capacity(tokens.len());
            let mut i = 0;
            while i < tokens.len() {
                if i < tokens.len() - 1 {
                    if let Some(&new_token) = self.bpe_merges.get(&(tokens[i], tokens[i + 1])) {
                        new_tokens.push(new_token);
                        i += 2;
                        merges_found = true;
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
            if !merges_found {
                break;
            }
        }

        let mut output_bytes = Vec::with_capacity(tokens.len() * 2);
        for token in tokens {
            output_bytes.extend_from_slice(&token.to_be_bytes());
        }
        Ok(output_bytes)
    }
}

// --- Basic Tokenization Strategy (New Default) ---

/// A tokenization strategy that converts each byte to a 16-bit token.
///
/// This strategy converts each input byte to a u16 token (256-511 range)
/// without applying any BPE merges. This provides true tokenization while
/// maintaining simplicity for users who don't need BPE compression.
pub struct BasicTokenizationStrategy;

#[async_trait::async_trait]
impl TokenizationStrategy for BasicTokenizationStrategy {
    #[instrument(skip(self, chunk_data), name = "basic_tokenization_strategy_process")]
    async fn process_chunk(&self, chunk_data: &[u8]) -> io::Result<Vec<u8>> {
        if chunk_data.is_empty() {
            return Ok(Vec::new());
        }

        debug!("Converting {} bytes to u16 tokens", chunk_data.len());

        // Convert each byte to u16 token (byte value range: 0-255)
        let mut output_bytes = Vec::with_capacity(chunk_data.len() * 2);
        for &byte in chunk_data {
            let token = byte as u16;
            output_bytes.extend_from_slice(&token.to_be_bytes());
        }

        Ok(output_bytes)
    }
}

// --- Passthrough Strategy Implementation (Explicit Copy Mode) ---

/// A tokenization strategy that performs no operations.
///
/// This strategy simply returns the input chunk as-is, acting as a no-op.
/// This is explicitly for file copying operations, not tokenization.
/// Use this only when you specifically want to copy files without any processing.
pub struct PassthroughStrategy;

#[async_trait::async_trait]
impl TokenizationStrategy for PassthroughStrategy {
    #[instrument(skip(self, chunk_data), name = "passthrough_strategy_process")]
    async fn process_chunk(&self, chunk_data: &[u8]) -> io::Result<Vec<u8>> {
        debug!(
            "Passthrough mode: returning {} bytes unchanged",
            chunk_data.len()
        );
        Ok(chunk_data.to_vec())
    }
}

// This module could later include:
// - Traits for different tokenization strategies.
// - Implementations for other strategies (e.g., patch-based).
// - Logic to select and apply strategies based on configuration.

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn u8_slice_to_u16_vec(slice: &[u8]) -> Vec<u16> {
        slice.iter().map(|&b| b as u16).collect()
    }

    fn u16_vec_to_byte_vec(tokens: &[u16]) -> Vec<u8> {
        tokens.iter().flat_map(|&t| t.to_be_bytes()).collect()
    }

    fn create_bpe_strategy(pairs: Vec<((u16, u16), u16)>) -> BpeStrategy {
        let bpe_merges = Arc::new(pairs.into_iter().collect());
        BpeStrategy::new(bpe_merges)
    }

    #[tokio::test]
    async fn test_bpe_strategy_no_merges() -> io::Result<()> {
        let strategy = create_bpe_strategy(vec![]);
        let chunk = b"abcdef";
        let expected_tokens = u8_slice_to_u16_vec(b"abcdef");

        let result = strategy.process_chunk(chunk).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_bpe_strategy_simple_merge() -> io::Result<()> {
        let strategy = create_bpe_strategy(vec![((97, 98), 256)]);
        let chunk = b"abcab";
        let expected_tokens = vec![256, 99, 256];

        let result = strategy.process_chunk(chunk).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_bpe_strategy_multiple_merges() -> io::Result<()> {
        let strategy = create_bpe_strategy(vec![((97, 98), 256), ((99, 100), 257)]);
        let chunk = b"abcdab";
        let expected_tokens = vec![256, 257, 256];

        let result = strategy.process_chunk(chunk).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_bpe_strategy_iterative_merging() -> io::Result<()> {
        let strategy = create_bpe_strategy(vec![((97, 98), 256), ((256, 99), 257)]);
        let chunk = b"abcde";
        let expected_tokens = vec![257, 100, 101];

        let result = strategy.process_chunk(chunk).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_bpe_strategy_no_possible_merges() -> io::Result<()> {
        let strategy = create_bpe_strategy(vec![((97, 98), 256)]);
        let chunk = b"xyz123";
        let expected_tokens = u8_slice_to_u16_vec(b"xyz123");

        let result = strategy.process_chunk(chunk).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_bpe_strategy_empty_input() -> io::Result<()> {
        let strategy = create_bpe_strategy(vec![((97, 98), 256)]);
        let chunk = b"";
        let expected_tokens: Vec<u16> = vec![];

        let result = strategy.process_chunk(chunk).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_bpe_strategy_single_byte_input_cannot_merge() -> io::Result<()> {
        let strategy = create_bpe_strategy(vec![((97, 98), 256)]);
        let chunk = b"a";
        let expected_tokens = vec![97u16];

        let result = strategy.process_chunk(chunk).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }

    #[tokio::test]
    async fn test_basic_tokenization_strategy() -> io::Result<()> {
        let strategy = BasicTokenizationStrategy;
        let chunk = b"abc";
        // 'a' = 97, 'b' = 98, 'c' = 99
        // As u16 big-endian bytes: [0, 97, 0, 98, 0, 99]
        let expected_bytes = vec![0, 97, 0, 98, 0, 99];

        let result = strategy.process_chunk(chunk).await?;
        assert_eq!(result, expected_bytes);
        Ok(())
    }

    #[tokio::test]
    async fn test_basic_tokenization_strategy_empty() -> io::Result<()> {
        let strategy = BasicTokenizationStrategy;
        let chunk = b"";
        let expected_bytes: Vec<u8> = vec![];

        let result = strategy.process_chunk(chunk).await?;
        assert_eq!(result, expected_bytes);
        Ok(())
    }

    #[tokio::test]
    async fn test_passthrough_strategy() -> io::Result<()> {
        let strategy = PassthroughStrategy;
        let chunk = b"ab c";
        let expected_bytes = chunk.to_vec();

        let result = strategy.process_chunk(chunk).await?;
        assert_eq!(result, expected_bytes);
        Ok(())
    }

    #[tokio::test]
    async fn test_bpe_strategy_merge_produces_byte_value() -> io::Result<()> {
        let strategy = create_bpe_strategy(vec![((120, 121), 90)]);
        let chunk = b"axyza";
        let expected_tokens = vec![97, 90, 122, 97];

        let result = strategy.process_chunk(chunk).await?;
        assert_eq!(result, u16_vec_to_byte_vec(&expected_tokens));
        Ok(())
    }
}
