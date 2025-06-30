use blt_core::{ContentType as CoreContentType, CoreConfig};
use clap::Parser;
use std::io;
use std::path::PathBuf;

// Default memory capacity percentage is now handled in blt_core

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, name = "blt")]
struct CliArgs {
    #[arg(short, long, value_name = "FILE", help = "Input file path (or - for stdin)")]
    input: Option<PathBuf>,

    #[arg(short, long, value_name = "FILE", help = "Output file path (or - for stdout)")]
    output: Option<PathBuf>,

    #[arg(long, value_name = "FILE", help = "BPE merges file for advanced tokenization")]
    merges: Option<PathBuf>,

    #[arg(long, help = "Use passthrough mode (copy file without tokenization)")]
    passthrough: bool,

    #[arg(long, value_enum, help = "Prepend content-type token")]
    r#type: Option<CliContentType>,

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

#[derive(clap::ValueEnum, Clone, Debug)]
enum CliContentType {
    Text,
    Audio,
    Bin,
    Video,
}

impl From<CliContentType> for CoreContentType {
    fn from(cli_type: CliContentType) -> Self {
        match cli_type {
            CliContentType::Text => CoreContentType::Text,
            CliContentType::Audio => CoreContentType::Audio,
            CliContentType::Bin => CoreContentType::Bin,
            CliContentType::Video => CoreContentType::Video,
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli_args = CliArgs::parse();

    let core_config = CoreConfig::new_from_cli(
        cli_args.input,
        cli_args.output,
        cli_args.merges,
        cli_args.r#type.map(CoreContentType::from),
        cli_args.threads,
        cli_args.chunksize,
        cli_args.memcap,
        cli_args.passthrough,
    )?;

    if let Err(e) = blt_core::run_tokenizer(core_config).await {
        eprintln!("Error running tokenizer: {e}");
        std::process::exit(1);
    }

    Ok(())
}
