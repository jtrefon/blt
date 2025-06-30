use std::fs::File; // Removed 'self'
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

// Helper to get the path to the compiled binary
fn get_cli_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push(if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    });
    path.push("blt"); // Name of the binary
    path
}

#[test]
fn test_cli_stdin_stdout() {
    let cli_path = get_cli_binary_path();
    let mut cmd = Command::new(cli_path);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn CLI process");
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(b"hello world")
            .expect("Failed to write to stdin");
    } // stdin is closed when it goes out of scope

    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());
    
    // Expected: each byte converted to u16 token in big-endian format
    let mut expected_output = Vec::new();
    for &byte in b"hello world" {
        expected_output.extend_from_slice(&(byte as u16).to_be_bytes());
    }
    assert_eq!(output.stdout, expected_output);
}

#[test]
fn test_cli_input_output_files() {
    let cli_path = get_cli_binary_path();

    let mut input_file = NamedTempFile::new().unwrap();
    input_file.write_all(b"hello from file").unwrap();
    let input_path = input_file.path();

    let output_file = NamedTempFile::new().unwrap();
    // let output_path = output_file.path(); // Unused variable

    // We need to drop output_file so that our CLI can write to it.
    // The NamedTempFile will be deleted when output_file_path_holder goes out of scope.
    let output_file_path_holder = output_file.into_temp_path();

    let mut cmd = Command::new(cli_path);
    cmd.arg("--input")
        .arg(input_path)
        .arg("--output")
        .arg(&output_file_path_holder);

    let status = cmd.status().expect("Failed to run CLI process");
    assert!(status.success());

    // Re-open the output file for reading
    let mut output_content = Vec::new();
    let mut f = File::open(&output_file_path_holder).unwrap();
    f.read_to_end(&mut output_content).unwrap();
    
    // Expected: each byte converted to u16 token in big-endian format
    let mut expected_output = Vec::new();
    for &byte in b"hello from file" {
        expected_output.extend_from_slice(&(byte as u16).to_be_bytes());
    }
    assert_eq!(output_content, expected_output);
}

#[test]
fn test_cli_type_argument() {
    let cli_path = get_cli_binary_path();
    let mut cmd = Command::new(cli_path);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
    cmd.arg("--type").arg("text");

    let mut child = cmd.spawn().expect("Failed to spawn CLI process");
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(b"test").expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());

    let mut expected_output = Vec::new();
    expected_output.extend_from_slice(&0xFF01u16.to_be_bytes()); // Text token
    // Each byte of "test" converted to u16 token
    for &byte in b"test" {
        expected_output.extend_from_slice(&(byte as u16).to_be_bytes());
    }
    assert_eq!(output.stdout, expected_output);
}

#[test]
fn test_cli_bpe_merges() {
    let cli_path = get_cli_binary_path();

    // Create a temporary merges file
    let mut merges_file = NamedTempFile::new().unwrap();
    merges_file.write_all(b"97 98\n").unwrap(); // 'a' 'b' -> 256
    let merges_path = merges_file.path();

    let mut cmd = Command::new(cli_path);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
    cmd.arg("--merges").arg(merges_path);

    let mut child = cmd.spawn().expect("Failed to spawn CLI process");
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(b"ab c ab")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());

    // Expected: 256 (ab), 32 (space as u16), 99 (c as u16), 32 (space as u16), 256 (ab)
    let mut expected_output = Vec::new();
    expected_output.extend_from_slice(&256u16.to_be_bytes());
    expected_output.extend_from_slice(&(b' ' as u16).to_be_bytes());
    expected_output.extend_from_slice(&(b'c' as u16).to_be_bytes());
    expected_output.extend_from_slice(&(b' ' as u16).to_be_bytes());
    expected_output.extend_from_slice(&256u16.to_be_bytes());

    assert_eq!(output.stdout, expected_output);
}

#[test]
fn test_cli_chunksize_argument() {
    // This test mainly checks if the argument is accepted and the program runs.
    // Verifying the exact chunking behavior internally is harder from outside.
    let cli_path = get_cli_binary_path();
    let mut cmd = Command::new(cli_path);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
    cmd.arg("--chunksize").arg("1KB");

    let mut child = cmd.spawn().expect("Failed to spawn CLI process");
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(b"some data")
            .expect("Failed to write to stdin");
    }
    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());
    
    // Expected: each byte converted to u16 token (basic tokenization)
    let mut expected_output = Vec::new();
    for &byte in b"some data" {
        expected_output.extend_from_slice(&(byte as u16).to_be_bytes());
    }
    assert_eq!(output.stdout, expected_output);
}

#[test]
fn test_cli_threads_argument() {
    // Similar to chunksize, mainly checks acceptance and successful run.
    let cli_path = get_cli_binary_path();
    let mut cmd = Command::new(cli_path);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
    cmd.arg("--threads").arg("1");

    let mut child = cmd.spawn().expect("Failed to spawn CLI process");
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(b"thread test")
            .expect("Failed to write to stdin");
    }
    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());
    
    // Expected: each byte converted to u16 token (basic tokenization)
    let mut expected_output = Vec::new();
    for &byte in b"thread test" {
        expected_output.extend_from_slice(&(byte as u16).to_be_bytes());
    }
    assert_eq!(output.stdout, expected_output);
}

#[test]
fn test_cli_passthrough_mode() {
    let cli_path = get_cli_binary_path();
    let mut cmd = Command::new(cli_path);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
    cmd.arg("--passthrough");

    let mut child = cmd.spawn().expect("Failed to spawn CLI process");
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(b"passthrough test")
            .expect("Failed to write to stdin");
    }
    let output = child.wait_with_output().expect("Failed to read stdout");
    assert!(output.status.success());
    
    // Passthrough mode should return the input unchanged
    assert_eq!(output.stdout, b"passthrough test");
}
