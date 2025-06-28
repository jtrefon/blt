// blt_core/src/io_handler.rs
// Handles input/output operations, including file access and stdin/stdout.

use crate::CoreConfig;
use tokio::fs::File as TokioFile;
use tokio::io::{AsyncRead, AsyncWrite, BufReader as TokioBufReader, BufWriter as TokioBufWriter};
use std::io;

// Type aliases for convenience
pub type InputReader = Box<dyn AsyncRead + Unpin + Send>;
pub type OutputWriter = Box<dyn AsyncWrite + Unpin + Send>;

pub async fn setup_input_reader(config: &CoreConfig) -> io::Result<InputReader> {
    match &config.input {
        Some(path) => {
            let file = TokioFile::open(path).await?;
            Ok(Box::new(TokioBufReader::new(file)))
        }
        None => Ok(Box::new(tokio::io::stdin())),
    }
}

pub async fn setup_output_writer(config: &CoreConfig) -> io::Result<OutputWriter> {
    match &config.output {
        Some(path) => {
            let file = TokioFile::create(path).await?;
            Ok(Box::new(TokioBufWriter::new(file)))
        }
        None => Ok(Box::new(tokio::io::stdout())),
    }
}

// Later, this module could include functions for managing ordered writing of processed chunks, etc.
// For example:
// pub async fn write_results_ordered(mut rx: tokio::sync::mpsc::Receiver<(usize, Vec<u8>)>, writer: &mut OutputWriter) -> io::Result<()> { ... }
