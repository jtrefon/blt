use clap::Parser;
use std::io;
use std::path::PathBuf;
use blt_core::{CoreConfig, ContentType as CoreContentType, utils, BpeMerges}; // Import utils
use std::sync::Arc;
// use std::collections::HashMap; // BpeMerges type is now from blt_core


// Default chunk size: 4MB - These defaults belong with CLI parsing
// const DEFAULT_CHUNK_SIZE_BYTES_STR: &str = "4MB"; // This is now handled by core if None
// Default memory capacity percentage
const DEFAULT_MEMCAP_PERCENT: u8 = 80;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, name = "blt")]
struct CliArgs {
    #[arg(short, long, value_name = "FILE")]
    input: Option<PathBuf>,

    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    #[arg(long, value_name = "FILE")]
    merges: Option<PathBuf>,

    #[arg(long, value_enum, help = "Prepend content-type token")]
    r#type: Option<CliContentType>, // Separate enum for CLI layer

    #[arg(long, value_name = "NUM", help = "Override worker count (default: auto based on cores-1)")]
    threads: Option<usize>,

    #[arg(long, value_name = "PERCENT", help = "Max RAM usage fraction (e.g., 70 for 70%)")]
    memcap: Option<u8>,

    #[arg(long, value_name = "SIZE", help = "Min/Max chunk size (e.g. 4MB, 256KB).")]
    chunksize: Option<String>,
}

// Enum for CLI parsing layer, to keep clap attributes separate from core logic
#[derive(clap::ValueEnum, Clone, Debug)]
enum CliContentType {
    Text,
    Audio,
    Bin,
}

// Conversion from CLI's ContentType to Core's ContentType
impl From<CliContentType> for CoreContentType {
    fn from(cli_type: CliContentType) -> Self {
        match cli_type {
            CliContentType::Text => CoreContentType::Text,
            CliContentType::Audio => CoreContentType::Audio,
            CliContentType::Bin => CoreContentType::Bin,
        }
    }
}

// Helper to load BPE merges, this might move into blt_core::config_loader later fully
// For now, main will do this to populate CoreConfig.bpe_data
fn load_bpe_merges_from_file(path: &PathBuf) -> io::Result<BpeMerges> {
    use std::fs::File;
    use std::io::BufReader;
    use std::io::BufRead;

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut merges = BpeMerges::new();
    let mut vocab_size = 256u16;

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 {
            let byte1 = parts[0].parse::<u8>().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse first byte: {}", e)))?;
            let byte2 = parts[1].parse::<u8>().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse second byte: {}", e)))?;
            merges.insert((byte1 as u16, byte2 as u16), vocab_size);
            vocab_size += 1;
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid merge rule format: expected two numbers separated by space"));
        }
    }
    Ok(merges)
}


#[tokio::main]
async fn main() -> io::Result<()> {
    let cli_args = CliArgs::parse();

    // Determine number of threads
    // N-1, or 1 if only 1 core. threads arg overrides.
    let num_threads = cli_args.threads.unwrap_or_else(|| {
        let cores = num_cpus::get();
        if cores > 1 {
            cores - 1
        } else {
            1
        }
    });

    // Parse chunk size if provided by user
    let cli_chunk_size: Option<usize> = cli_args.chunksize
        .map(|cs_str| utils::parse_chunk_size_str(&cs_str))
        .transpose()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    // Load BPE data if merges file is provided
    let bpe_data_arc: Option<Arc<BpeMerges>> = match cli_args.merges {
        Some(ref path) => {
            let merges = load_bpe_merges_from_file(path)?;
            Some(Arc::new(merges))
        }
        None => None,
    };


    // Construct CoreConfig
    let core_config = CoreConfig {
        input: cli_args.input,
        output: cli_args.output,
        merges_file: cli_args.merges, // Keep original path for reference if needed by core
        content_type: cli_args.r#type.map(CoreContentType::from),
        num_threads,
        cli_chunk_size, // Pass the Option<usize>
        mem_cap_percent: cli_args.memcap.unwrap_or(DEFAULT_MEMCAP_PERCENT),
        bpe_data: bpe_data_arc,
    };

    // Run the core tokenizer logic
    if let Err(e) = blt_core::run_tokenizer(core_config).await {
        eprintln!("Error running tokenizer: {}", e);
        // Consider exiting with a non-zero status code
        std::process::exit(1);
    }

    Ok(())
}
