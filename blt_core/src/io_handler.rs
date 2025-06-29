// blt_core/src/io_handler.rs
// Handles input/output operations, including file access and stdin/stdout.

//! Handles all input and output operations for the tokenizer.
//!
//! This module provides the logic for setting up input sources and output sinks. It
//! abstracts away the differences between file-based I/O and standard I/O streams
//! (stdin/stdout). A key feature is its ability to use memory-mapped files for
//! efficient processing of file inputs.

use crate::CoreConfig;
use memmap2::Mmap;
use std::fs::File;
use std::io;
use tokio::io::{AsyncRead, AsyncWrite, BufWriter as TokioBufWriter};

// --- Type Aliases for I/O ---

/// A type alias for a readable, asynchronous input stream.
pub type InputReader = Box<dyn AsyncRead + Unpin + Send>;
/// A type alias for a writable, asynchronous output stream.
pub type OutputWriter = Box<dyn AsyncWrite + Unpin + Send>;

// --- Public Enums and Functions ---

/// Represents the source of input data for the pipeline.
///
/// This enum allows the pipeline to seamlessly handle different kinds of input:
/// - A memory-mapped file (`Mmap`), which offers the highest performance for file-based input
///   by avoiding extra copying.
/// - A standard input stream (`Stdin`), for piping data into the application.
pub enum InputSource {
    /// A memory-mapped file.
    Mmap(Mmap),
    /// An asynchronous reader for standard input.
    Stdin(InputReader),
}

/// Sets up the input source and output writer based on the provided configuration.
///
/// This is the primary entry point for the I/O handler. It interprets the `CoreConfig`
/// to determine whether to read from a file or stdin, and whether to write to a file or
/// stdout.
///
/// # Arguments
/// * `config` - A reference to the `CoreConfig` containing I/O settings.
///
/// # Returns
/// A `Result` containing a tuple of `(InputSource, OutputWriter)` on success, or an
/// `io::Error` on failure.
pub async fn setup_io(config: &CoreConfig) -> io::Result<(InputSource, OutputWriter)> {
    let input_source = match &config.input {
        Some(path) => {
            let file = File::open(path)?;
            let mmap = unsafe { Mmap::map(&file)? };
            InputSource::Mmap(mmap)
        }
        None => {
            let stdin_reader = Box::new(tokio::io::stdin());
            InputSource::Stdin(stdin_reader)
        }
    };

    let output_writer = setup_output_writer(config).await?;
    Ok((input_source, output_writer))
}

async fn setup_output_writer(config: &CoreConfig) -> io::Result<OutputWriter> {
    match &config.output {
        Some(path) => {
            let file = tokio::fs::File::create(path).await?;
            Ok(Box::new(TokioBufWriter::new(file)))
        }
        None => Ok(Box::new(tokio::io::stdout())),
    }
}

// Later, this module could include functions for managing ordered writing of processed chunks, etc.
// For example:
// pub async fn write_results_ordered(mut rx: tokio::sync::mpsc::Receiver<(usize, Vec<u8>)>, writer: &mut OutputWriter) -> io::Result<()> { ... }
