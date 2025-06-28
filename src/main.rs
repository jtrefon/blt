use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File as TokioFile;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

// Default chunk size: 4MB
const DEFAULT_CHUNK_SIZE_BYTES: usize = 4 * 1024 * 1024;
// Default memory capacity percentage
const DEFAULT_MEMCAP_PERCENT: u8 = 80;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    input: Option<PathBuf>,

    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    #[arg(long, value_name = "FILE")]
    merges: Option<PathBuf>,

    #[arg(long, value_enum, help = "Prepend content-type token")]
    r#type: Option<ContentType>,

    #[arg(long, value_name = "NUM", help = "Override worker count (default: detected cores)")]
    threads: Option<usize>,

    #[arg(long, value_name = "PERCENT", help = "Max RAM usage fraction (e.g., 70 for 70%)")]
    memcap: Option<u8>,

    #[arg(long, value_name = "SIZE", help = "Min/Max chunk size (e.g. 4MB, 256KB). For now, sets the chunk size directly.")]
    chunksize: Option<String>, // Parses into usize later
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum ContentType {
    Text,
    Audio,
    Bin,
}

impl ContentType {
    // Example token values. These would be part of the defined vocabulary.
    // For now, using arbitrary high values to avoid collision with byte values or simple BPE merges.
    fn get_token_value(&self) -> u16 {
        match self {
            ContentType::Text => 0xFF01,
            ContentType::Audio => 0xFF02,
            ContentType::Bin => 0xFF03,
        }
    }
}


use std::path::Path; // Import Path

type BpeMerges = HashMap<(u16, u16), u16>;

fn parse_chunk_size(s: &str) -> Result<usize, String> {
    let s_trimmed = s.trim();
    if s_trimmed.is_empty() {
        return Err("Input string is empty".to_string());
    }

    let s_upper = s_trimmed.to_uppercase();

    let (num_part, unit_part) = if s_upper.ends_with("KB") {
        match s_upper.char_indices().rev().nth(1) { // Get second to last char index
            Some((idx, _)) => s_trimmed.split_at(idx),
            None => return Err(format!("Invalid format: {}", s)), // Should not happen if ends_with KB
        }
    } else if s_upper.ends_with("MB") {
        match s_upper.char_indices().rev().nth(1) { // Get second to last char index
            Some((idx, _)) => s_trimmed.split_at(idx),
            None => return Err(format!("Invalid format: {}", s)), // Should not happen if ends_with MB
        }
    } else if s_upper.chars().all(|c| c.is_digit(10)) {
        (s_trimmed, "") // No unit, all digits
    } else {
        // Check if it's a number followed by non-KB/MB chars or mixed
        let last_digit_pos = s_trimmed.rfind(|c: char| c.is_digit(10));
        if let Some(pos) = last_digit_pos {
            let (num_candidate, unit_candidate) = s_trimmed.split_at(pos + 1);
            if unit_candidate.is_empty() && num_candidate.chars().all(|c| c.is_digit(10)) {
                 (num_candidate, "") // All digits confirmed
            } else {
                // It could be something like "1024X" which is invalid
                return Err(format!("Invalid unit or format: {}. Use KB, MB, or raw bytes.", s));
            }
        } else {
            // No digits found, e.g. "abc" or "MB"
            return Err(format!("Invalid number part: {}. Use KB, MB, or raw bytes.", s));
        }
    };

    let num = num_part.parse::<usize>().map_err(|_| format!("Invalid number: {}", num_part))?;

    match unit_part.to_uppercase().as_str() {
        "KB" => Ok(num * 1024),
        "MB" => Ok(num * 1024 * 1024),
        "" => Ok(num), // Raw bytes
        _ => Err(format!("Invalid unit: {}. Use KB, MB, or raw bytes.", unit_part)),
    }
}


fn load_bpe_merges(path: &Path) -> io::Result<BpeMerges> { // Changed to &Path
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut merges = BpeMerges::new();
    let mut vocab_size = 256u16;

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("#") || line.is_empty() {
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

async fn process_chunk(chunk: Vec<u8>, bpe_merges_opt: Option<Arc<BpeMerges>>) -> io::Result<Vec<u8>> {
    if bpe_merges_opt.is_none() {
        // If no BPE, output is still u8 bytes.
        return Ok(chunk);
    }
    let merges = bpe_merges_opt.unwrap();
    // Initial sequence of u16 tokens (bytes promoted to u16)
    let mut tokens: Vec<u16> = chunk.into_iter().map(|b| b as u16).collect();

    loop {
        let mut new_tokens = Vec::with_capacity(tokens.len());
        let mut i = 0;
        let mut merged_this_pass = false;

        while i < tokens.len() {
            if i + 1 < tokens.len() {
                if let Some(&new_token) = merges.get(&(tokens[i], tokens[i + 1])) {
                    new_tokens.push(new_token);
                    i += 2;
                    merged_this_pass = true;
                } else {
                    new_tokens.push(tokens[i]);
                    i += 1;
                }
            } else {
                new_tokens.push(tokens[i]);
                i += 1;
            }
        }
        tokens = new_tokens;
        if !merged_this_pass {
            break;
        }
    }

    // Convert u16 tokens back to Vec<u8> for output (big-endian).
    let mut output_bytes = Vec::with_capacity(tokens.len() * 2);
    for token in tokens {
        output_bytes.extend_from_slice(&token.to_be_bytes());
    }
    Ok(output_bytes)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let bpe_merges_arc: Option<Arc<BpeMerges>> = match cli.merges {
        Some(ref pathbuf) => Some(Arc::new(load_bpe_merges(pathbuf.as_path())?)), // Pass as &Path
        None => None,
    };

    let mut input_reader: Box<dyn tokio::io::AsyncRead + Unpin> = match cli.input {
        Some(path) => Box::new(TokioFile::open(path).await?),
        None => Box::new(tokio::io::stdin()),
    };

    let mut output_writer: Box<dyn tokio::io::AsyncWrite + Unpin> = match cli.output {
        Some(path) => Box::new(TokioFile::create(path).await?),
        None => Box::new(tokio::io::stdout()),
    };

    let num_workers = cli.threads.unwrap_or_else(num_cpus::get);

    let chunk_size_bytes = match cli.chunksize {
        Some(cs_str) => parse_chunk_size(&cs_str).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?,
        None => DEFAULT_CHUNK_SIZE_BYTES,
    };

    let _memcap_percent = cli.memcap.unwrap_or(DEFAULT_MEMCAP_PERCENT);
    // Actual memory capping logic is not implemented in this step.
    // eprintln!("Using {} worker threads.", num_workers);
    // eprintln!("Using chunk size: {} bytes.", chunk_size_bytes);
    // eprintln!("Memory cap: {}% (not strictly enforced yet).", _memcap_percent);


    // Handle --type: Prepend content-type token if specified
    if let Some(content_type) = cli.r#type {
        let type_token_val = content_type.get_token_value();
        // Outputting as u16 BE bytes, similar to BPE tokens
        output_writer.write_all(&type_token_val.to_be_bytes()).await?;
    }


    let (results_tx, mut results_rx) = mpsc::channel::<(usize, io::Result<Vec<u8>>)>(num_workers * 2);

    let mut next_chunk_id: usize = 0;
    let mut dispatched_task_handles: HashMap<usize, JoinHandle<()>> = HashMap::new();
    let mut received_results: HashMap<usize, io::Result<Vec<u8>>> = HashMap::new();
    let mut current_expected_chunk_id: usize = 0;
    let mut input_eof = false;

    loop {
        while !input_eof && dispatched_task_handles.len() < num_workers {
            let mut chunk_buffer = vec![0; chunk_size_bytes]; // Use configurable chunk_size
            let bytes_read = input_reader.read(&mut chunk_buffer).await?;

            if bytes_read == 0 {
                input_eof = true;
                break;
            }
            chunk_buffer.truncate(bytes_read);

            let task_id = next_chunk_id;
            next_chunk_id += 1;

            let current_bpe_ref = bpe_merges_arc.clone();
            let results_tx_clone = results_tx.clone();

            let handle = tokio::spawn(async move {
                let result = process_chunk(chunk_buffer, current_bpe_ref).await;
                if results_tx_clone.send((task_id, result)).await.is_err() {
                    // eprintln!("Error sending result for task {}: receiver dropped.", task_id);
                }
            });
            dispatched_task_handles.insert(task_id, handle);
        }

        if dispatched_task_handles.is_empty() && input_eof {
            break;
        }

        tokio::select! {
            biased;
            maybe_result = results_rx.recv(), if !dispatched_task_handles.is_empty() || input_eof => {
                match maybe_result {
                    Some((task_id, result)) => {
                        dispatched_task_handles.remove(&task_id);
                        received_results.insert(task_id, result);
                    }
                    None => { // Channel disconnected
                        break;
                    }
                }
            }
        }

        while let Some(result_data) = received_results.remove(&current_expected_chunk_id) {
            match result_data {
                Ok(chunk_data) => output_writer.write_all(&chunk_data).await?,
                Err(e) => {
                    eprintln!("Error in processed chunk {}: {:?}", current_expected_chunk_id, e);
                    return Err(e);
                }
            }
            current_expected_chunk_id += 1;
        }

        if input_eof && dispatched_task_handles.is_empty() && received_results.is_empty() {
            break;
        }
    }

    drop(results_tx);

    while let Some((task_id, result)) = results_rx.recv().await {
        received_results.insert(task_id, result);
        while let Some(result_data) = received_results.remove(&current_expected_chunk_id) {
            match result_data {
                Ok(chunk_data) => output_writer.write_all(&chunk_data).await?,
                Err(e) => {
                     eprintln!("Error in processed chunk {} (final collection): {:?}", current_expected_chunk_id, e);
                    return Err(e);
                }
            }
            current_expected_chunk_id += 1;
        }
    }

    while let Some(result_data) = received_results.remove(&current_expected_chunk_id) {
        match result_data {
            Ok(chunk_data) => output_writer.write_all(&chunk_data).await?,
            Err(e) => {
                eprintln!("Error in processed chunk {} (very final collection): {:?}", current_expected_chunk_id, e);
                return Err(e);
            }
        }
        current_expected_chunk_id += 1;
    }

    output_writer.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*; // Make items from outer scope available

    #[test]
    fn test_parse_chunk_size_valid() {
        assert_eq!(parse_chunk_size("1024"), Ok(1024));
        assert_eq!(parse_chunk_size("1kb"), Ok(1024));
        assert_eq!(parse_chunk_size("1KB"), Ok(1024));
        assert_eq!(parse_chunk_size("2mb"), Ok(2 * 1024 * 1024));
        assert_eq!(parse_chunk_size("2MB"), Ok(2 * 1024 * 1024));
        assert_eq!(parse_chunk_size("10MB"), Ok(10 * 1024 * 1024));
    }

    #[test]
    fn test_parse_chunk_size_invalid() {
        assert!(parse_chunk_size("1gb").is_err());
        assert!(parse_chunk_size("mb1").is_err());
        assert!(parse_chunk_size("1024b").is_err());
        assert!(parse_chunk_size("").is_err());
        assert!(parse_chunk_size("abc").is_err());
        assert!(parse_chunk_size("10.5MB").is_err());
    }

    use tempfile::TempPath;

    // Helper to create a temporary BPE merges file
    fn create_temp_merges_file(content: &str) -> io::Result<TempPath> {
        use std::io::Write;
        let mut temp_file = tempfile::NamedTempFile::new()?;
        temp_file.write_all(content.as_bytes())?;
        Ok(temp_file.into_temp_path())
    }

    #[test]
    fn test_load_bpe_merges_valid() {
        let content = "# Merge rule for 'a' 'b'\n97 98\n# Merge rule for 'c' 'd'\n99 100\n\n101 102";
        let temp_file_holder = create_temp_merges_file(content).unwrap(); // Renamed to indicate it holds the TempPath

        let merges = load_bpe_merges(temp_file_holder.as_ref()).unwrap(); // Pass Path via as_ref()

        let mut expected = BpeMerges::new();
        expected.insert((97u16, 98u16), 256u16);  // a b -> 256
        expected.insert((99u16, 100u16), 257u16); // c d -> 257
        expected.insert((101u16, 102u16), 258u16);// e f -> 258

        assert_eq!(merges, expected);
        // tempfile automatically cleans up
    }

    #[test]
    fn test_load_bpe_merges_invalid_format() {
        let content = "97 98 99"; // Too many parts
        let temp_file_holder = create_temp_merges_file(content).unwrap();
        assert!(load_bpe_merges(temp_file_holder.as_ref()).is_err());
    }

    #[test]
    fn test_load_bpe_merges_invalid_byte() {
        let content = "97 300"; // 300 is not a u8
        let temp_file_holder = create_temp_merges_file(content).unwrap();
        assert!(load_bpe_merges(temp_file_holder.as_ref()).is_err());
    }

    #[tokio::test]
    async fn test_process_chunk_no_bpe() {
        let chunk = vec![1, 2, 3, 4, 5];
        let processed = process_chunk(chunk.clone(), None).await.unwrap();
        assert_eq!(processed, chunk);
    }

    #[tokio::test]
    async fn test_process_chunk_with_bpe() {
        let mut merges_map = BpeMerges::new();
        merges_map.insert((1u16, 2u16), 256u16); // 1 2 -> 256
        merges_map.insert((256u16, 3u16), 257u16); // (1 2) 3 -> 257
        let bpe_merges = Some(Arc::new(merges_map));

        let chunk = vec![1, 2, 3, 4, 5, 1, 2];
        // Expected: (1 2) -> 256. Sequence: 256, 3, 4, 5, 256
        // Next pass: (256 3) -> 257. Sequence: 257, 4, 5, 256
        let processed = process_chunk(chunk, bpe_merges).await.unwrap();

        let mut expected_bytes = Vec::new();
        expected_bytes.extend_from_slice(&257u16.to_be_bytes()); // 257
        expected_bytes.extend_from_slice(&4u16.to_be_bytes());   // 4 (as u16)
        expected_bytes.extend_from_slice(&5u16.to_be_bytes());   // 5 (as u16)
        expected_bytes.extend_from_slice(&256u16.to_be_bytes()); // 256

        assert_eq!(processed, expected_bytes);
    }
     #[tokio::test]
    async fn test_process_chunk_with_bpe_no_match() {
        let mut merges_map = BpeMerges::new();
        merges_map.insert((10u16, 20u16), 256u16);
        let bpe_merges = Some(Arc::new(merges_map));

        let chunk = vec![1, 2, 3, 4, 5];
        // Expected: 1, 2, 3, 4, 5 (each as u16 BE bytes)
        let processed = process_chunk(chunk.clone(), bpe_merges).await.unwrap();

        let mut expected_bytes = Vec::new();
        for &byte_val in chunk.iter() {
            expected_bytes.extend_from_slice(&(byte_val as u16).to_be_bytes());
        }
        assert_eq!(processed, expected_bytes);
    }
}
