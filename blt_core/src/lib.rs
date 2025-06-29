//! # Byte-Level Tokenizer Core Library (blt_core)
//!
//! The `blt_core` crate orchestrates the entire tokenization process. It is designed for high
//! performance and flexibility, supporting multiple tokenization strategies and efficient,
//! concurrent I/O handling.
//!
//! ## Core Concepts
//!
//! - **Configuration (`CoreConfig`):** A central struct that holds all operational parameters,
//!   from input/output paths to threading and memory settings.
//! - **Pipeline (`pipeline::run`):** The heart of the tokenizer. It processes data in chunks,
//!   leveraging an asynchronous, multi-threaded architecture. It supports both memory-mapped files
//!   for maximum efficiency and streaming input for flexibility.
//! - **Tokenization Strategy (`tokenizer::TokenizationStrategy`):** A trait that allows for
//!   pluggable tokenization algorithms. The two primary strategies are `BpeStrategy` for
//!   Byte-Pair Encoding and `PassthroughStrategy` for no-op tokenization.
//! - **I/O Handling (`io_handler`):** Manages input sources (files, stdin) and output sinks
//!   (files, stdout), abstracting away the details of synchronous vs. asynchronous I/O.
//!
//! ## Example Usage
//!
//! ```no_run
//! use blt_core::{CoreConfig, run_tokenizer};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = CoreConfig::new_from_cli(
//!         Some(PathBuf::from("input.txt")),
//!         Some(PathBuf::from("output.bin")),
//!         None, // No BPE merges, use passthrough
//!         None,
//!         None,
//!         None,
//!         None,
//!     ).unwrap();
//!
//!     if let Err(e) = run_tokenizer(config).await {
//!         eprintln!("Error: {}", e);
//!     }
//! }
//! ```

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tracing::{info, instrument};

use crate::tokenizer::{BpeStrategy, PassthroughStrategy, TokenizationStrategy};

// --- Module declarations ---
/// Handles dynamic chunk sizing based on system memory and CLI parameters.
pub mod chunking;
/// Responsible for loading BPE merge files.
pub mod config_loader;
/// Manages input and output sources, supporting files and standard I/O.
pub mod io_handler;
/// Contains the core multi-threaded pipeline logic for processing data chunks.
pub mod pipeline;
/// Defines tokenization strategies (BPE, Passthrough) and the `TokenizationStrategy` trait.
pub mod tokenizer;
/// Utilities for parsing configurations and detecting system resources.
pub mod utils;

// --- Public API ---

/// A type alias for the BPE merge map.
///
/// The map consists of a pair of tokens (as `u16`) that can be merged into a single new token (`u16`).
pub type BpeMerges = HashMap<(u16, u16), u16>;

/// Represents the type of content being processed.
///
/// This enum is used to prepend a special token to the output stream, allowing downstream
/// consumers to identify the nature of the original content.
#[derive(Clone, Debug, PartialEq)]
pub enum ContentType {
    /// Plain text content.
    Text,
    /// Audio data.
    Audio,
    /// Generic binary data.
    Bin,
    /// Video data.
    Video,
}

impl ContentType {
    /// Returns the special token value associated with each content type.
    /// These tokens are in a reserved range (0xFF01 - 0xFF04).
    pub fn get_token_value(&self) -> u16 {
        match self {
            ContentType::Text => 0xFF01,
            ContentType::Audio => 0xFF02,
            ContentType::Bin => 0xFF03,
            ContentType::Video => 0xFF04,
        }
    }
}

/// Central configuration for the tokenizer pipeline.
///
/// This struct holds all the necessary settings to control the tokenization process,
/// including I/O paths, tokenization strategy, and performance tuning parameters.
#[derive(Debug, Clone)]
pub struct CoreConfig {
    /// Path to the input file. If `None`, stdin will be used.
    pub input: Option<PathBuf>,
    /// Path to the output file. If `None`, stdout will be used.
    pub output: Option<PathBuf>,
    /// Path to the BPE merges file. If `None`, the passthrough strategy is used.
    pub merges_file: Option<PathBuf>,
    /// The type of content being processed.
    pub content_type: Option<ContentType>,
    /// The number of threads to use for the processing pipeline.
    pub num_threads: usize,
    /// The chunk size specified via CLI, in bytes.
    pub cli_chunk_size: Option<usize>,
    /// The percentage of system RAM to use as a cap for the chunk size.
    pub mem_cap_percent: u8,
    /// Pre-loaded BPE merge data.
    pub bpe_data: Option<Arc<BpeMerges>>,
}

impl CoreConfig {
    /// Creates a new `CoreConfig` from command-line arguments.
    ///
    /// This is the primary entry point for creating a configuration. It handles parsing,
    /// validation, and loading of necessary resources like BPE merge files.
    ///
    /// # Arguments
    ///
    /// * `input`: Optional path to the input file.
    /// * `output`: Optional path to the output file.
    /// * `merges`: Optional path to the BPE merges file.
    /// * `content_type`: Optional `ContentType` of the input.
    /// * `threads`: Optional number of threads to use.
    /// * `chunksize`: Optional chunk size as a string (e.g., "16MB").
    /// * `memcap`: Optional memory capacity percentage.
    pub fn new_from_cli(
        input: Option<PathBuf>,
        output: Option<PathBuf>,
        merges: Option<PathBuf>,
        content_type: Option<ContentType>,
        threads: Option<usize>,
        chunksize: Option<String>,
        memcap: Option<u8>,
    ) -> io::Result<Self> {
        let num_threads = utils::determine_thread_count(threads);
        let cli_chunk_size = Self::parse_chunksize(chunksize)?;
        let bpe_data = Self::load_bpe_data(&merges)?;

        Ok(CoreConfig {
            input,
            output,
            merges_file: merges,
            content_type,
            num_threads,
            cli_chunk_size,
            mem_cap_percent: memcap.unwrap_or(80),
            bpe_data,
        })
    }

    fn parse_chunksize(chunksize: Option<String>) -> io::Result<Option<usize>> {
        chunksize
            .as_ref()
            .map(|cs_str| utils::parse_chunk_size_str(cs_str))
            .transpose()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
    }

    fn load_bpe_data(merges_path: &Option<PathBuf>) -> io::Result<Option<Arc<BpeMerges>>> {
        match merges_path {
            Some(path) => {
                let merges_map = Self::load_merges_from_file(path)?;
                Ok(Some(Arc::new(merges_map)))
            }
            None => Ok(None),
        }
    }

    fn load_merges_from_file(path: &Path) -> io::Result<BpeMerges> {
        config_loader::load_bpe_merges_from_path(path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Failed to load BPE merges: {e}"),
            )
        })
    }
}

/// Loads BPE merges from a file path.
///
/// This function loads BPE merge rules from a file and returns them as a HashMap.
/// The file should contain pairs of bytes that can be merged, one pair per line.
///
/// # Arguments
///
/// * `path`: Path to the BPE merges file.
///
/// # Returns
///
/// A HashMap mapping byte pairs to new token IDs.
pub fn load_bpe_merges(path: &Path) -> io::Result<HashMap<(u8, u8), u16>> {
    let merges = config_loader::load_bpe_merges_from_path(path)?;
    // Convert from (u16, u16) to (u8, u8) for Python compatibility
    let converted: HashMap<(u8, u8), u16> = merges
        .into_iter()
        .filter_map(|((a, b), token)| {
            if a <= 255 && b <= 255 {
                Some(((a as u8, b as u8), token))
            } else {
                None
            }
        })
        .collect();
    Ok(converted)
}

/// Runs the entire tokenization pipeline with the given configuration.
///
/// This is the main entry point of the `blt_core` library. It sets up the I/O,
/// selects the tokenization strategy, and launches the processing pipeline.
///
/// # Arguments
///
/// * `config`: A `CoreConfig` struct containing all the necessary settings.
///
/// # Errors
///
/// This function can return an `io::Error` if there are issues with file I/O,
/// configuration loading, or during the processing pipeline itself.
#[instrument(skip_all, fields(input = ?config.input, output = ?config.output))]
pub async fn run_tokenizer(config: CoreConfig) -> io::Result<()> {
    info!("Starting tokenizer");

    let strategy = select_strategy(&config);
    let effective_chunk_size = chunking::get_effective_chunk_size(&config);
    info!(effective_chunk_size, "Chunk size determined");

    let (input_source, mut output_writer) = io_handler::setup_io(&config).await?;
    prepend_content_type_token(&mut output_writer, config.content_type.as_ref()).await?;

    pipeline::run(
        input_source,
        output_writer,
        effective_chunk_size,
        config.num_threads,
        strategy,
    )
    .await?;

    info!("Tokenizer run completed successfully");
    Ok(())
}

// --- Private Helper Functions ---

fn select_strategy(config: &CoreConfig) -> Arc<dyn TokenizationStrategy> {
    if let Some(ref bpe_data) = config.bpe_data {
        info!("Using BPE tokenization strategy.");
        Arc::new(BpeStrategy::new(bpe_data.clone()))
    } else {
        info!("Using passthrough tokenization strategy.");
        Arc::new(PassthroughStrategy)
    }
}

async fn prepend_content_type_token(
    writer: &mut io_handler::OutputWriter,
    content_type: Option<&ContentType>,
) -> io::Result<()> {
    if let Some(ct) = content_type {
        writer
            .write_all(&ct.get_token_value().to_be_bytes())
            .await?;
    }
    Ok(())
}
