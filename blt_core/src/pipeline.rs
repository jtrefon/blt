//! This module is internal to `blt_core` and contains the core concurrent processing pipeline.
//!
//! It is not intended for direct use by external crates. The public-in-private functions
//! are exposed to the rest of the crate but not as part of the public API.
//!
//! # Tokenization Pipeline
//!
//! This module contains the core concurrent processing pipeline for the tokenizer.
//! It handles reading from an input source, spawning parallel tasks for tokenization,
//! and writing the ordered results to an output sink.

use crate::io_handler::{self, InputSource, OutputWriter};
use crate::tokenizer::TokenizationStrategy;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tracing::{debug, error, info, info_span, instrument, Instrument};

/// The main entry point for running the tokenization pipeline.
#[instrument(skip_all, name = "run_pipeline")]
pub(crate) async fn run(
    input_source: InputSource,
    output_writer: OutputWriter,
    effective_chunk_size: usize,
    num_threads: usize,
    strategy: Arc<dyn TokenizationStrategy>,
) -> io::Result<()> {
    match input_source {
        InputSource::Mmap(mmap) => {
            run_mmap_pipeline(
                mmap,
                output_writer,
                effective_chunk_size,
                num_threads,
                strategy,
            )
            .await
        }
        InputSource::Stdin(input_reader) => {
            run_stream_pipeline(
                input_reader,
                output_writer,
                effective_chunk_size,
                num_threads,
                strategy,
            )
            .await
        }
    }
}

// --- Mmap Pipeline ---

async fn run_mmap_pipeline(
    mmap: memmap2::Mmap,
    mut output_writer: OutputWriter,
    effective_chunk_size: usize,
    num_threads: usize,
    strategy: Arc<dyn TokenizationStrategy>,
) -> io::Result<()> {
    info!(
        "Running pipeline in Mmap mode for file of size: {}",
        mmap.len()
    );
    let mmap_arc = Arc::new(mmap);
    let (results_tx, mut results_rx) = mpsc::channel(num_threads * 2);
    let mut dispatched_task_handles = HashMap::new();
    let mut received_results = HashMap::new();
    let mut current_expected_chunk_id = 0;

    let chunks: Vec<(usize, usize)> = mmap_arc
        .chunks(effective_chunk_size)
        .enumerate()
        .map(|(i, chunk)| {
            let start = i * effective_chunk_size;
            let len = chunk.len();
            (start, len)
        })
        .collect();

    let mut chunk_iter = chunks.into_iter().enumerate();

    loop {
        while dispatched_task_handles.len() < num_threads {
            if let Some((task_id, (start, len))) = chunk_iter.next() {
                let handle = spawn_mmap_chunk_task(
                    task_id,
                    mmap_arc.clone(),
                    start,
                    len,
                    strategy.clone(),
                    results_tx.clone(),
                )
                .await;
                dispatched_task_handles.insert(task_id, handle);
            } else {
                break;
            }
        }

        if dispatched_task_handles.is_empty() {
            break;
        }

        if let Some((task_id, result)) = results_rx.recv().await {
            debug!(task_id, "Received result for mmap task");
            dispatched_task_handles.remove(&task_id);
            received_results.insert(task_id, result);
            write_ordered_mmap_results(
                &mut received_results,
                &mut current_expected_chunk_id,
                &mut output_writer,
            )
            .await?;
        } else {
            break;
        }
    }

    finalize_mmap_results(
        &mut received_results,
        &mut current_expected_chunk_id,
        &mut output_writer,
    )
    .await?;

    output_writer.flush().await?;
    Ok(())
}

async fn spawn_mmap_chunk_task(
    task_id: usize,
    mmap_arc: Arc<memmap2::Mmap>,
    start: usize,
    len: usize,
    strategy: Arc<dyn TokenizationStrategy>,
    results_tx: mpsc::Sender<(usize, io::Result<Vec<u8>>)>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(
        async move {
            let chunk_slice = &mmap_arc[start..start + len];
            let result = strategy.process_chunk(chunk_slice).await;
            if results_tx.send((task_id, result)).await.is_err() {
                error!(task_id, "Failed to send mmap result: receiver dropped.");
            }
        }
        .instrument(info_span!("process_mmap_chunk_task", task_id)),
    )
}

async fn write_ordered_mmap_results(
    received_results: &mut HashMap<usize, io::Result<Vec<u8>>>,
    current_expected_chunk_id: &mut usize,
    output_writer: &mut OutputWriter,
) -> io::Result<()> {
    while let Some(result_data) = received_results.remove(current_expected_chunk_id) {
        match result_data {
            Ok(chunk_data) => {
                output_writer.write_all(&chunk_data).await?;
            }
            Err(e) => return Err(e),
        }
        *current_expected_chunk_id += 1;
    }
    Ok(())
}

async fn finalize_mmap_results(
    received_results: &mut HashMap<usize, io::Result<Vec<u8>>>,
    current_expected_chunk_id: &mut usize,
    output_writer: &mut OutputWriter,
) -> io::Result<()> {
    let mut sorted_keys: Vec<usize> = received_results.keys().copied().collect();
    sorted_keys.sort_unstable();

    for key in sorted_keys {
        if key == *current_expected_chunk_id {
            if let Some(result_data) = received_results.remove(&key) {
                match result_data {
                    Ok(chunk_data) => {
                        output_writer.write_all(&chunk_data).await?;
                    }
                    Err(e) => return Err(e),
                }
                *current_expected_chunk_id += 1;
            }
        }
    }
    Ok(())
}

// --- Stream Pipeline (for Stdin) ---

async fn run_stream_pipeline(
    mut input_reader: io_handler::InputReader,
    mut output_writer: OutputWriter,
    effective_chunk_size: usize,
    num_threads: usize,
    strategy: Arc<dyn TokenizationStrategy>,
) -> io::Result<()> {
    info!("Running pipeline in Stream mode for stdin");
    let (results_tx, mut results_rx) = mpsc::channel(num_threads * 2);
    let mut context = ProcessingContext::new();

    loop {
        manage_task_spawning(
            &mut context,
            &mut input_reader,
            effective_chunk_size,
            num_threads,
            strategy.clone(),
            results_tx.clone(),
        )
        .await?;

        if context.is_work_done() {
            break;
        }

        if context.no_tasks_running_and_input_available() {
            continue;
        }

        if await_and_process_task_result(&mut context, &mut results_rx, &mut output_writer).await? {
            break;
        }

        if context.is_all_work_truly_done() {
            break;
        }
    }

    drop(results_tx);

    finalize_results(&mut context, &mut results_rx, &mut output_writer).await?;
    output_writer.flush().await?;
    Ok(())
}

// --- Private Structs and Functions ---

/// Holds the state for the duration of the processing loop.
struct ProcessingContext {
    next_chunk_id: usize,
    dispatched_task_handles: HashMap<usize, tokio::task::JoinHandle<()>>,
    received_results: HashMap<usize, io::Result<Vec<u8>>>,
    current_expected_chunk_id: usize,
    input_eof: bool,
}

impl ProcessingContext {
    fn new() -> Self {
        Self {
            next_chunk_id: 0,
            dispatched_task_handles: HashMap::new(),
            received_results: HashMap::new(),
            current_expected_chunk_id: 0,
            input_eof: false,
        }
    }
    fn is_work_done(&self) -> bool {
        self.input_eof && self.dispatched_task_handles.is_empty()
    }
    fn no_tasks_running_and_input_available(&self) -> bool {
        self.dispatched_task_handles.is_empty() && !self.input_eof
    }
    fn is_all_work_truly_done(&self) -> bool {
        self.input_eof
            && self.dispatched_task_handles.is_empty()
            && self.received_results.is_empty()
    }
}

/// Fills the worker pool with new tasks as long as there is capacity and input.
#[instrument(skip_all)]
async fn manage_task_spawning(
    context: &mut ProcessingContext,
    input_reader: &mut io_handler::InputReader,
    effective_chunk_size: usize,
    num_threads: usize,
    strategy: Arc<dyn TokenizationStrategy>,
    results_tx_clone: mpsc::Sender<(usize, io::Result<Vec<u8>>)>,
) -> io::Result<()> {
    while !context.input_eof && context.dispatched_task_handles.len() < num_threads {
        if !try_read_and_spawn_task(
            context,
            input_reader,
            effective_chunk_size,
            strategy.clone(),
            results_tx_clone.clone(),
        )
        .await?
        {
            break;
        }
    }
    Ok(())
}

/// Reads a single chunk and spawns a processing task for it.
async fn try_read_and_spawn_task(
    context: &mut ProcessingContext,
    input_reader: &mut io_handler::InputReader,
    effective_chunk_size: usize,
    strategy: Arc<dyn TokenizationStrategy>,
    results_tx: mpsc::Sender<(usize, io::Result<Vec<u8>>)>,
) -> io::Result<bool> {
    let mut chunk_buffer = vec![0; effective_chunk_size];
    let bytes_read = input_reader.read(&mut chunk_buffer).await?;

    if bytes_read == 0 {
        context.input_eof = true;
        debug!("Input stream reached EOF");
        return Ok(false);
    }
    chunk_buffer.truncate(bytes_read);

    let task_id = context.next_chunk_id;
    context.next_chunk_id += 1;

    debug!(
        task_id,
        bytes = bytes_read,
        "Spawning chunk processing task"
    );
    let handle = spawn_chunk_processing_task(task_id, chunk_buffer, strategy, results_tx);
    context.dispatched_task_handles.insert(task_id, handle);
    Ok(true)
}

/// Spawns a Tokio task to process a single chunk.
#[instrument(skip_all)]
fn spawn_chunk_processing_task(
    task_id: usize,
    chunk_buffer: Vec<u8>,
    strategy: Arc<dyn TokenizationStrategy>,
    results_tx: mpsc::Sender<(usize, io::Result<Vec<u8>>)>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(
        async move {
            let result = strategy.process_chunk(&chunk_buffer).await;
            if results_tx.send((task_id, result)).await.is_err() {
                error!(task_id, "Failed to send result: receiver dropped.");
            }
        }
        .instrument(info_span!("process_chunk_task", task_id)),
    )
}

/// Waits for a task result and processes it. Returns `true` if the main loop should break.
async fn await_and_process_task_result(
    context: &mut ProcessingContext,
    results_rx: &mut mpsc::Receiver<(usize, io::Result<Vec<u8>>)>,
    output_writer: &mut OutputWriter,
) -> io::Result<bool> {
    tokio::select! {
        biased;
        maybe_result = results_rx.recv(), if !context.dispatched_task_handles.is_empty() || context.input_eof => {
            return process_received_results(context, maybe_result, output_writer).await;
        }
        else => {
            Ok(false)
        }
    }
}

/// Handles a received result from a task. Returns `true` if the main loop should break.
async fn process_received_results(
    context: &mut ProcessingContext,
    maybe_result: Option<(usize, io::Result<Vec<u8>>)>,
    output_writer: &mut OutputWriter,
) -> io::Result<bool> {
    match maybe_result {
        Some((task_id, result)) => {
            debug!(task_id, "Received result for task");
            context.dispatched_task_handles.remove(&task_id);
            context.received_results.insert(task_id, result);
        }
        None => {
            debug!("Result channel disconnected, ending processing loop");
            return Ok(true);
        }
    }
    write_ordered_results(context, output_writer).await?;
    Ok(false)
}

/// Writes any completed and ordered chunks to the output.
async fn write_ordered_results(
    context: &mut ProcessingContext,
    output_writer: &mut OutputWriter,
) -> io::Result<()> {
    while let Some(result_data) = context
        .received_results
        .remove(&context.current_expected_chunk_id)
    {
        match result_data {
            Ok(chunk_data) => {
                debug!(
                    chunk_id = context.current_expected_chunk_id,
                    bytes = chunk_data.len(),
                    "Writing ordered chunk to output"
                );
                output_writer.write_all(&chunk_data).await?
            }
            Err(e) => {
                error!(
                    chunk_id = context.current_expected_chunk_id,
                    "Error in processed chunk: {:?}", e
                );
                return Err(e);
            }
        }
        context.current_expected_chunk_id += 1;
    }
    Ok(())
}

/// Ensures any remaining results in the channel or context are processed and written.
async fn finalize_results(
    context: &mut ProcessingContext,
    results_rx: &mut mpsc::Receiver<(usize, io::Result<Vec<u8>>)>,
    output_writer: &mut OutputWriter,
) -> io::Result<()> {
    while let Some((task_id, result)) = results_rx.recv().await {
        context.received_results.insert(task_id, result);
        write_ordered_results(context, output_writer).await?;
    }
    write_ordered_results(context, output_writer).await?; // Final check
    Ok(())
}
