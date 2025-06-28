use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::mpsc; // For passing results from tasks
use tokio::io::{AsyncReadExt, AsyncWriteExt}; // Added AsyncReadExt

// Re-export or define necessary types
pub use crate::utils::parse_chunk_size_str; // Re-export for main.rs if it still needs it (it does for CLI parsing)
pub type BpeMerges = HashMap<(u16, u16), u16>;

#[derive(Clone, Debug)]
pub enum ContentType {
    Text, Audio, Bin,
}

impl ContentType {
    pub fn get_token_value(&self) -> u16 {
        match self {
            ContentType::Text => 0xFF01,
            ContentType::Audio => 0xFF02,
            ContentType::Bin => 0xFF03,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CoreConfig {
    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub merges_file: Option<PathBuf>, // For reference; actual merges in bpe_data
    pub content_type: Option<ContentType>,
    pub num_threads: usize,
    pub cli_chunk_size: Option<usize>,
    pub mem_cap_percent: u8,
    pub bpe_data: Option<Arc<BpeMerges>>,
}

// --- Module declarations ---
pub mod chunking;
pub mod config_loader;
pub mod io_handler;
pub mod tokenizer;
pub mod utils;

// --- Helper functions for run_tokenizer ---

async fn initialize_io(
    config: &CoreConfig
) -> io::Result<(io_handler::InputReader, io_handler::OutputWriter)> {
    let input_reader = io_handler::setup_input_reader(config).await?;
    let output_writer = io_handler::setup_output_writer(config).await?;
    Ok((input_reader, output_writer))
}

async fn prepend_content_type_token(
    writer: &mut io_handler::OutputWriter,
    content_type: Option<&ContentType>,
) -> io::Result<()> {
    if let Some(ct) = content_type {
        writer.write_all(&ct.get_token_value().to_be_bytes()).await?;
    }
    Ok(())
}

// Structure to hold state for the processing loop
struct ProcessingContext {
    next_chunk_id: usize,
    dispatched_task_handles: HashMap<usize, tokio::task::JoinHandle<()>>,
    received_results: HashMap<usize, io::Result<Vec<u8>>>,
    current_expected_chunk_id: usize,
    input_eof: bool,
}

impl ProcessingContext {
    fn new() -> Self {
        ProcessingContext {
            next_chunk_id: 0,
            dispatched_task_handles: HashMap::new(),
            received_results: HashMap::new(),
            current_expected_chunk_id: 0,
            input_eof: false,
        }
    }
}

async fn spawn_chunk_processing_task(
    task_id: usize,
    chunk_buffer: Vec<u8>,
    bpe_data: Option<Arc<BpeMerges>>,
    results_tx: mpsc::Sender<(usize, io::Result<Vec<u8>>)>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let result = tokenizer::process_chunk(chunk_buffer, bpe_data).await;
        if results_tx.send((task_id, result)).await.is_err() {
            // eprintln!("Error sending result for task {}: receiver dropped.", task_id);
        }
    })
}

async fn manage_task_spawning_and_input_reading(
    context: &mut ProcessingContext,
    input_reader: &mut io_handler::InputReader,
    effective_chunk_size: usize,
    num_threads: usize,
    bpe_data: Option<Arc<BpeMerges>>,
    results_tx_clone: mpsc::Sender<(usize, io::Result<Vec<u8>>)>,
) -> io::Result<()> {
    while !context.input_eof && context.dispatched_task_handles.len() < num_threads {
        let mut chunk_buffer = vec![0; effective_chunk_size];
        let bytes_read = input_reader.read(&mut chunk_buffer).await?;
        if bytes_read == 0 { context.input_eof = true; break; }
        chunk_buffer.truncate(bytes_read);

        let task_id = context.next_chunk_id;
        context.next_chunk_id += 1;
        let handle = spawn_chunk_processing_task(
            task_id, chunk_buffer, bpe_data.clone(), results_tx_clone.clone()
        ).await;
        context.dispatched_task_handles.insert(task_id, handle);
    }
    Ok(())
}

async fn process_received_results(
    context: &mut ProcessingContext,
    maybe_result: Option<(usize, io::Result<Vec<u8>>)>,
    output_writer: &mut io_handler::OutputWriter,
) -> io::Result<bool> { // Returns true if loop should break
    match maybe_result {
        Some((task_id, result)) => {
            context.dispatched_task_handles.remove(&task_id);
            context.received_results.insert(task_id, result);
        }
        None => return Ok(true), // Channel disconnected, break loop
    }
    write_ordered_results(context, output_writer).await?;
    Ok(false) // Don't break loop yet
}

async fn write_ordered_results(
    context: &mut ProcessingContext,
    output_writer: &mut io_handler::OutputWriter,
) -> io::Result<()> {
    while let Some(result_data) = context.received_results.remove(&context.current_expected_chunk_id) {
        match result_data {
            Ok(chunk_data) => output_writer.write_all(&chunk_data).await?,
            Err(e) => { /* eprintln!("Error in processed chunk {}: {:?}", context.current_expected_chunk_id, e); */ return Err(e); }
        }
        context.current_expected_chunk_id += 1;
    }
    Ok(())
}

async fn main_processing_loop(
    config: &CoreConfig,
    input_reader: &mut io_handler::InputReader,
    output_writer: &mut io_handler::OutputWriter,
    effective_chunk_size: usize,
) -> io::Result<()> {
    let (results_tx, mut results_rx) = mpsc::channel(config.num_threads * 2);
    let mut context = ProcessingContext::new();

    loop {
        manage_task_spawning_and_input_reading(
            &mut context, input_reader, effective_chunk_size, config.num_threads,
            config.bpe_data.clone(), results_tx.clone()
        ).await?;

        if context.dispatched_task_handles.is_empty() && context.input_eof { break; }

        tokio::select! {
            biased;
            maybe_result = results_rx.recv(), if !context.dispatched_task_handles.is_empty() || context.input_eof => {
                if process_received_results(&mut context, maybe_result, output_writer).await? { break; }
            }
        }
        if context.input_eof && context.dispatched_task_handles.is_empty() && context.received_results.is_empty() { break; }
    }
    drop(results_tx); // Close the sender side of the channel
    finalize_results(&mut context, &mut results_rx, output_writer).await?;
    Ok(())
}

async fn finalize_results(
    context: &mut ProcessingContext,
    results_rx: &mut mpsc::Receiver<(usize, io::Result<Vec<u8>>)>,
    output_writer: &mut io_handler::OutputWriter,
) -> io::Result<()> {
    while let Some((task_id, result)) = results_rx.recv().await {
        context.received_results.insert(task_id, result);
        write_ordered_results(context, output_writer).await?;
    }
    write_ordered_results(context, output_writer).await?; // Final check
    Ok(())
}

// Main entry point for the core tokenizer logic
pub async fn run_tokenizer(config: CoreConfig) -> io::Result<()> {
    let effective_chunk_size = chunking::get_effective_chunk_size(&config);
    // println!("Effective chunk size to be used: {} bytes", effective_chunk_size);

    let (mut input_reader, mut output_writer) = initialize_io(&config).await?;
    prepend_content_type_token(&mut output_writer, config.content_type.as_ref()).await?;

    main_processing_loop(&config, &mut input_reader, &mut output_writer, effective_chunk_size).await?;

    output_writer.flush().await?; // Ensure all buffered data is written
    Ok(())
}
