//! # Chunking Logic for BLT
//!
//! This module determines the appropriate size for data chunks that are processed
//! in parallel by the tokenizer. The goal is to balance memory usage, CPU utilization,
//! and I/O throughput.
//!
//! Chunk size can be specified by the user via CLI arguments or calculated
//! dynamically based on available system RAM and the number of processing threads.

use crate::CoreConfig;
use sysinfo::System; // Removed SystemExt from direct import

// Default chunk sizes if not specified by user and dynamic calculation fails or is bounded.
const DEFAULT_MIN_CHUNK_SIZE_BYTES: usize = 1024 * 1024; // 1MB (1 * 1024 * 1024)
const DEFAULT_MAX_CHUNK_SIZE_BYTES: usize = 16 * 1024 * 1024; // 16MB
const ABSOLUTE_MIN_CHUNK_SIZE: usize = 256 * 1024; // 256KB, absolute floor
const ABSOLUTE_MAX_CHUNK_SIZE: usize = 128 * 1024 * 1024; // 128MB, absolute ceiling for auto-calc

/// Determines the effective chunk size to use for processing.
/// If `config.cli_chunk_size` is Some, it's used directly (respecting absolute min/max).
/// Otherwise, dynamically calculates based on system RAM and number of threads.
pub fn get_effective_chunk_size(config: &CoreConfig) -> usize {
    if let Some(cli_size) = config.cli_chunk_size {
        // User specified a chunk size, use that, but clamp it reasonably.
        return cli_size.clamp(ABSOLUTE_MIN_CHUNK_SIZE, ABSOLUTE_MAX_CHUNK_SIZE);
    }

    // Dynamic calculation based on system resources
    let mut sys = System::new_all();
    sys.refresh_memory(); // Refresh RAM info

    // Total system RAM in bytes
    let total_ram_bytes = sys.total_memory();

    // Memory available for token buffers (e.g., 80% of total RAM, as per mem_cap_percent)
    // Convert mem_cap_percent (u8) to f64 for calculation
    let usable_ram_for_buffers =
        (total_ram_bytes as f64 * (config.mem_cap_percent as f64 / 100.0)) as u64;

    // Divide usable RAM by number of threads to get per-thread RAM budget.
    // Add a buffer factor (e.g., 2) because each chunk might be held in memory
    // by the reader, the processor, and potentially the writer before flushing.
    // So, each "active" chunk might need 2-3x its size in RAM across stages.
    // Let's aim for each thread to comfortably handle one chunk in its pipeline stage.
    // A more conservative approach: RAM per thread / buffer_factor (e.g. 2 or 3)
    let ram_per_thread_budget = usable_ram_for_buffers / (config.num_threads as u64);

    // Tentative chunk size based on RAM per thread.
    // Let's use a buffer factor of, say, 4 to be conservative, meaning a chunk
    // should ideally not exceed 1/4th of the RAM budget allocated per thread.
    // This accounts for potential copies, intermediate states, and other overhead.
    let calculated_chunk_size = (ram_per_thread_budget / 4) as usize;

    // Clamp the dynamically calculated chunk size to sensible defaults and absolute limits.
    calculated_chunk_size
        .clamp(DEFAULT_MIN_CHUNK_SIZE_BYTES, DEFAULT_MAX_CHUNK_SIZE_BYTES)
        .clamp(ABSOLUTE_MIN_CHUNK_SIZE, ABSOLUTE_MAX_CHUNK_SIZE)
}

// This function is a placeholder from before, we'll remove or integrate it.
// pub fn calculate_chunk_size(config: &CoreConfig, total_ram_gb: f32) -> usize {
//     println!("[chunking] Calculating chunk size. RAM: {}GB, Threads: {}, MemCap: {}%, Configured ChunkSize: {:?}",
//         total_ram_gb, config.num_threads, config.mem_cap_percent, config.cli_chunk_size);
//     // config.cli_chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE_BYTES) // old logic
//     get_effective_chunk_size(config) // New logic
// }

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::{ContentType, BpeMerges}; // Currently not needed for these specific tests
    // use std::sync::Arc; // Currently not needed for these specific tests

    fn create_test_config(
        cli_chunk_size: Option<usize>,
        num_threads: usize,
        mem_cap_percent: u8,
    ) -> CoreConfig {
        CoreConfig {
            input: None,
            output: None,
            merges_file: None,
            content_type: None,
            num_threads,
            cli_chunk_size,
            mem_cap_percent,
            bpe_data: None,
        }
    }

    #[test]
    fn test_get_effective_chunk_size_cli_override() {
        let config = create_test_config(Some(5 * 1024 * 1024), 4, 80);
        assert_eq!(get_effective_chunk_size(&config), 5 * 1024 * 1024);

        // Test clamping with CLI override
        let config_too_small = create_test_config(Some(10 * 1024), 4, 80); // 10KB
        assert_eq!(
            get_effective_chunk_size(&config_too_small),
            ABSOLUTE_MIN_CHUNK_SIZE
        );

        let config_too_large = create_test_config(Some(200 * 1024 * 1024), 4, 80); // 200MB
        assert_eq!(
            get_effective_chunk_size(&config_too_large),
            ABSOLUTE_MAX_CHUNK_SIZE
        );
    }

    #[test]
    fn test_get_effective_chunk_size_dynamic() {
        // This test is environment-dependent (relies on actual system RAM).
        // We can't assert an exact value, but we can check if it's within bounds.
        let config = create_test_config(None, 4, 80); // Auto, 4 threads, 80% mem cap
        let dynamic_size = get_effective_chunk_size(&config);

        println!("Dynamic chunk size calculated: {} bytes", dynamic_size);

        assert!(dynamic_size >= DEFAULT_MIN_CHUNK_SIZE_BYTES.min(ABSOLUTE_MIN_CHUNK_SIZE));
        assert!(dynamic_size <= DEFAULT_MAX_CHUNK_SIZE_BYTES.max(ABSOLUTE_MAX_CHUNK_SIZE));
        assert!(dynamic_size >= ABSOLUTE_MIN_CHUNK_SIZE);
        assert!(dynamic_size <= ABSOLUTE_MAX_CHUNK_SIZE);

        // Example with low memory cap to force lower end of clamp
        let config_low_mem_cap = create_test_config(None, 4, 1); // 1% mem cap
        let dynamic_size_low_mem = get_effective_chunk_size(&config_low_mem_cap);
        println!(
            "Dynamic chunk size (low mem cap): {} bytes",
            dynamic_size_low_mem
        );
        // It should clamp to DEFAULT_MIN_CHUNK_SIZE or ABSOLUTE_MIN_CHUNK_SIZE
        assert!(dynamic_size_low_mem <= DEFAULT_MAX_CHUNK_SIZE_BYTES);
        assert!(dynamic_size_low_mem >= ABSOLUTE_MIN_CHUNK_SIZE);

        // Example with many threads to force lower chunk size
        let config_many_threads = create_test_config(None, 128, 80); // 128 threads
        let dynamic_size_many_threads = get_effective_chunk_size(&config_many_threads);
        println!(
            "Dynamic chunk size (many threads): {} bytes",
            dynamic_size_many_threads
        );
        assert!(dynamic_size_many_threads <= DEFAULT_MAX_CHUNK_SIZE_BYTES);
        assert!(dynamic_size_many_threads >= ABSOLUTE_MIN_CHUNK_SIZE);
    }
}
