use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc; // For passing results from tasks // Added AsyncReadExt

// Re-export or define necessary types
pub use crate::utils::parse_chunk_size_str; // Re-export for main.rs if it still needs it (it does for CLI parsing)
pub type BpeMerges = HashMap<(u16, u16), u16>;

#[derive(Clone, Debug)]
pub enum ContentType {
    Text,
    Audio,
    Bin,
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
    config: &CoreConfig,
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
        writer
            .write_all(&ct.get_token_value().to_be_bytes())
            .await?;
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

/// Helper function to read a chunk from input and spawn a processing task.
/// Returns `Ok(true)` if a task was spawned, `Ok(false)` if EOF was reached.
async fn try_read_and_spawn_task(
    context: &mut ProcessingContext,
    input_reader: &mut io_handler::InputReader,
    effective_chunk_size: usize,
    bpe_data: Option<Arc<BpeMerges>>,
    results_tx: mpsc::Sender<(usize, io::Result<Vec<u8>>)>,
) -> io::Result<bool> {
    let mut chunk_buffer = vec![0; effective_chunk_size];
    let bytes_read = input_reader.read(&mut chunk_buffer).await?;

    if bytes_read == 0 {
        context.input_eof = true;
        return Ok(false); // EOF reached, no task spawned
    }
    chunk_buffer.truncate(bytes_read);

    let task_id = context.next_chunk_id;
    context.next_chunk_id += 1;

    let handle =
        spawn_chunk_processing_task(task_id, chunk_buffer, bpe_data.clone(), results_tx.clone())
            .await;
    context.dispatched_task_handles.insert(task_id, handle);
    Ok(true) // Task spawned
}

async fn manage_task_spawning_and_input_reading(
    context: &mut ProcessingContext,
    input_reader: &mut io_handler::InputReader,
    effective_chunk_size: usize,
    num_threads: usize, // Controls how many tasks can be in flight
    bpe_data: Option<Arc<BpeMerges>>,
    results_tx_clone: mpsc::Sender<(usize, io::Result<Vec<u8>>)>,
) -> io::Result<()> {
    // Fill the worker pool as long as there's capacity and no EOF
    while !context.input_eof && context.dispatched_task_handles.len() < num_threads {
        if !try_read_and_spawn_task(
            context,
            input_reader,
            effective_chunk_size,
            bpe_data.clone(),
            results_tx_clone.clone(),
        )
        .await?
        {
            // false means EOF was reached by try_read_and_spawn_task
            break;
        }
        // If true, a task was spawned; loop continues if capacity allows.
    }
    Ok(())
}

async fn process_received_results(
    context: &mut ProcessingContext,
    maybe_result: Option<(usize, io::Result<Vec<u8>>)>,
    output_writer: &mut io_handler::OutputWriter,
) -> io::Result<bool> {
    // Returns true if loop should break
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
    while let Some(result_data) = context
        .received_results
        .remove(&context.current_expected_chunk_id)
    {
        match result_data {
            Ok(chunk_data) => output_writer.write_all(&chunk_data).await?,
            Err(e) => {
                /* eprintln!("Error in processed chunk {}: {:?}", context.current_expected_chunk_id, e); */
                return Err(e);
            }
        }
        context.current_expected_chunk_id += 1;
    }
    Ok(())
}

async fn await_and_process_task_result(
    context: &mut ProcessingContext,
    results_rx: &mut mpsc::Receiver<(usize, io::Result<Vec<u8>>)>,
    output_writer: &mut io_handler::OutputWriter,
) -> io::Result<bool> {
    // Returns true if the main loop should break due to channel close or error
    // This select will only proceed if there are active tasks or if EOF has been reached (to drain pending results).
    // The `if` condition on `results_rx.recv()` is crucial.
    tokio::select! {
        biased; // Process received results first if available.
        maybe_result = results_rx.recv(), if !context.dispatched_task_handles.is_empty() || context.input_eof => {
            // process_received_results returns Ok(true) if the channel was closed, signaling a break.
            // It returns Ok(false) if a result was processed normally.
            // It returns Err if writing failed.
            return process_received_results(context, maybe_result, output_writer).await;
        }
        // else => {
            // This else branch would be hit if the condition on results_rx.recv() is false.
            // This means no tasks are dispatched AND input is not EOF.
            // In this scenario, the main loop should continue to try spawning more tasks.
            // So, we don't want to break here.
        // }
        else => {
            // This branch is taken if the condition on results_rx.recv() is false.
            // This means: context.dispatched_task_handles.is_empty() && !context.input_eof.
            // In this situation, the main loop should attempt to spawn more tasks, so we don't break.
            Ok(false) // Removed redundant return
        }
    }
    // This line is now truly unreachable due to the `else` branch also returning/being an expression.
    // Ok(false)
}

async fn main_processing_loop(
    config: &CoreConfig,
    input_reader: &mut io_handler::InputReader,
    output_writer: &mut io_handler::OutputWriter,
    effective_chunk_size: usize,
) -> io::Result<()> {
    let (results_tx, mut results_rx) = mpsc::channel(config.num_threads * 2); // Buffer for results
    let mut context = ProcessingContext::new();

    loop {
        // Attempt to spawn new tasks if there's capacity and input available.
        manage_task_spawning_and_input_reading(
            &mut context,
            input_reader,
            effective_chunk_size,
            config.num_threads,
            config.bpe_data.clone(),
            results_tx.clone(), // Clone sender for each task spawner iteration
        )
        .await?;

        // Condition 1: If all input has been read (EOF) and all dispatched tasks have finished processing
        // (i.e., their results are either in `received_results` or have been written),
        // then we can break the loop. `finalize_results` will handle any remaining ordered writes.
        if context.input_eof && context.dispatched_task_handles.is_empty() {
            break;
        }

        // If there are no tasks running and input is not yet EOF, we should continue to the next iteration
        // to spawn more tasks. Avoid calling `await_and_process_task_result` if it would block indefinitely.
        if context.dispatched_task_handles.is_empty() && !context.input_eof {
            // This case should ideally be handled by manage_task_spawning filling up workers,
            // but as a safeguard, ensure we loop back to try spawning if nothing is running.
            continue;
        }

        // Await and process at least one task result.
        // If `await_and_process_task_result` returns true, it means the results channel closed or an error occurred,
        // so the main loop should terminate.
        if await_and_process_task_result(&mut context, &mut results_rx, output_writer).await? {
            break; // Break if channel closed or error in processing/writing
        }

        // Condition 2: After processing a result, check again if all work is truly done.
        // This handles the case where processing the last result makes everything complete.
        if context.input_eof
            && context.dispatched_task_handles.is_empty()
            && context.received_results.is_empty()
        {
            break;
        }
    }

    drop(results_tx); // Explicitly drop the sender to close the channel.

    // Ensure any remaining results in the channel or context are processed and written.
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

    main_processing_loop(
        &config,
        &mut input_reader,
        &mut output_writer,
        effective_chunk_size,
    )
    .await?;

    output_writer.flush().await?; // Ensure all buffered data is written
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_get_token_value() {
        assert_eq!(ContentType::Text.get_token_value(), 0xFF01);
        assert_eq!(ContentType::Audio.get_token_value(), 0xFF02);
        assert_eq!(ContentType::Bin.get_token_value(), 0xFF03);
    }

    // Add more tests for lib.rs specific logic if any parts can be unit tested in isolation.
    // For example, testing parts of the ProcessingContext or specific state transitions if they were public
    // and didn't solely rely on private fields or heavy async machinery.
    // For now, the main run_tokenizer loop is best tested via integration tests.
}
