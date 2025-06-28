use blt_core::{config_loader, utils, BpeMerges, ContentType as CoreContentType, CoreConfig}; // Added config_loader
use clap::Parser;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

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

    #[arg(
        long,
        value_name = "NUM",
        help = "Override worker count (default: auto based on cores)"
    )]
    threads: Option<usize>,

    #[arg(
        long,
        value_name = "PERCENT",
        help = "Max RAM usage fraction (e.g., 70 for 70%)"
    )]
    memcap: Option<u8>,

    #[arg(
        long,
        value_name = "SIZE",
        help = "Min/Max chunk size (e.g. 4MB, 256KB)."
    )]
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

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli_args = CliArgs::parse();

    // Determine number of threads using the utility function from blt_core
    let num_threads = utils::determine_thread_count(cli_args.threads);

    // Parse chunk size if provided by user
    let cli_chunk_size: Option<usize> = cli_args
        .chunksize
        .map(|cs_str| utils::parse_chunk_size_str(&cs_str))
        .transpose()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    // Load BPE data if merges file is provided, using the function from blt_core::config_loader
    let bpe_data_arc: Option<Arc<BpeMerges>> = match cli_args.merges {
        Some(ref path) => {
            // Use the config_loader function directly
            let merges = config_loader::load_bpe_merges_from_path(path).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Failed to load BPE merges: {}", e),
                )
            })?;
            Some(Arc::new(merges))
        }
        None => None,
    };

    // Construct CoreConfig
    let core_config = CoreConfig {
        input: cli_args.input,
        output: cli_args.output,
        merges_file: cli_args.merges, // Keep original path for reference
        content_type: cli_args.r#type.map(CoreContentType::from),
        num_threads,
        cli_chunk_size,
        mem_cap_percent: cli_args.memcap.unwrap_or(DEFAULT_MEMCAP_PERCENT),
        bpe_data: bpe_data_arc,
    };

    // Run the core tokenizer logic
    if let Err(e) = blt_core::run_tokenizer(core_config).await {
        eprintln!("Error running tokenizer: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
