// blt_core/src/tokenizer.rs
// Contains tokenization strategies (BPE, patching, etc.).

use crate::BpeMerges; // Using the type alias from lib.rs
use std::io;
use std::sync::Arc;

// This is the core BPE processing logic, adapted from the previous main.rs
pub async fn process_chunk_bpe(
    chunk_data: Vec<u8>,
    bpe_merges: Arc<BpeMerges> // Assuming BPE merges are always provided if this fn is called
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
