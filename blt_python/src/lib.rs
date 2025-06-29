use blt_core::{run_tokenizer, ContentType, CoreConfig};
use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// A Python wrapper for the BLT tokenizer.
///
/// This class provides a high-level interface to the Rust-based BLT tokenizer,
/// allowing Python applications to leverage high-performance byte-level tokenization.
///
/// # Examples
///
/// Basic usage:
/// ```python
/// import blt
/// tokenizer = blt.ByteTokenizer()
/// tokenizer.tokenize_file("input.txt", "output.bin")
/// ```
///
/// With BPE merges:
/// ```python
/// merges = {(97, 98): 256}  # 'a' + 'b' -> token 256
/// tokenizer = blt.ByteTokenizer(merges=merges)
/// tokenizer.tokenize_file("input.txt", "output.bin")
/// ```
#[pyclass]
pub struct ByteTokenizer {
    merges: Option<HashMap<(u8, u8), u16>>,
    content_type: Option<String>,
    threads: Option<usize>,
    chunk_size: Option<String>,
    memory_cap: Option<u8>,
}

#[pymethods]
impl ByteTokenizer {
    /// Create a new ByteTokenizer instance.
    ///
    /// # Arguments
    ///
    /// * `merges` - Optional dictionary of BPE merges: {(byte1, byte2): new_token}
    /// * `content_type` - Optional content type hint ("Text" or "Bin")
    /// * `threads` - Optional number of processing threads
    /// * `chunk_size` - Optional chunk size (e.g., "1MB", "512KB")
    /// * `memory_cap` - Optional memory usage cap as percentage (0-100)
    #[new]
    #[pyo3(signature = (merges=None, content_type=None, threads=None, chunk_size=None, memory_cap=None))]
    pub fn new(
        merges: Option<HashMap<(u8, u8), u16>>,
        content_type: Option<String>,
        threads: Option<usize>,
        chunk_size: Option<String>,
        memory_cap: Option<u8>,
    ) -> PyResult<Self> {
        // Validate memory_cap
        if let Some(cap) = memory_cap {
            if cap > 100 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "memory_cap must be between 0 and 100",
                ));
            }
        }

        // Validate content_type
        if let Some(ref ct) = content_type {
            match ct.as_str() {
                "Text" | "Bin" => {}
                _ => {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "content_type must be 'Text' or 'Bin'",
                    ))
                }
            }
        }

        Ok(ByteTokenizer {
            merges,
            content_type,
            threads,
            chunk_size,
            memory_cap,
        })
    }

    /// Tokenize a file and write the result to another file.
    ///
    /// # Arguments
    ///
    /// * `input_path` - Path to the input file
    /// * `output_path` - Path to the output file
    ///
    /// # Raises
    ///
    /// * `RuntimeError` - If tokenization fails
    /// * `IOError` - If file operations fail
    pub fn tokenize_file(&self, input_path: &str, output_path: &str) -> PyResult<()> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to create async runtime: {}",
                e
            ))
        })?;

        rt.block_on(async {
            // Create temporary file for merges if we have them
            let _temp_file = if let Some(ref merges) = self.merges {
                let temp_file = tempfile::NamedTempFile::new().map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                        "Failed to create temporary file: {}",
                        e
                    ))
                })?;

                // Write merges to temporary file
                use std::io::Write;
                {
                    let mut file = std::fs::File::create(temp_file.path()).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                            "Failed to write merges file: {}",
                            e
                        ))
                    })?;

                    for ((a, b), _token) in merges {
                        writeln!(file, "{} {}", a, b).map_err(|e| {
                            PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                                "Failed to write merge: {}",
                                e
                            ))
                        })?;
                    }
                }

                // Create configuration and run tokenization
                let content_type = self.content_type.as_ref().and_then(|ct| match ct.as_str() {
                    "Text" => Some(ContentType::Text),
                    "Bin" => Some(ContentType::Bin),
                    _ => None,
                });

                let config = CoreConfig::new_from_cli(
                    Some(PathBuf::from(input_path)),
                    Some(PathBuf::from(output_path)),
                    Some(temp_file.path().to_path_buf()),
                    content_type,
                    self.threads,
                    self.chunk_size.clone(),
                    self.memory_cap,
                )
                .map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Failed to create configuration: {}",
                        e
                    ))
                })?;

                run_tokenizer(config).await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Tokenization failed: {}",
                        e
                    ))
                })?;

                Some(temp_file)
            } else {
                // Create configuration and run tokenization without merges
                let content_type = self.content_type.as_ref().and_then(|ct| match ct.as_str() {
                    "Text" => Some(ContentType::Text),
                    "Bin" => Some(ContentType::Bin),
                    _ => None,
                });

                let config = CoreConfig::new_from_cli(
                    Some(PathBuf::from(input_path)),
                    Some(PathBuf::from(output_path)),
                    None,
                    content_type,
                    self.threads,
                    self.chunk_size.clone(),
                    self.memory_cap,
                )
                .map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Failed to create configuration: {}",
                        e
                    ))
                })?;

                run_tokenizer(config).await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "Tokenization failed: {}",
                        e
                    ))
                })?;

                None
            };

            // Keep temp file alive until this point
            drop(_temp_file);
            Ok::<(), PyErr>(())
        })?;

        Ok(())
    }

    /// String representation of the tokenizer configuration.
    fn __repr__(&self) -> String {
        format!(
            "ByteTokenizer(merges={}, content_type={:?}, threads={:?}, chunk_size={:?}, memory_cap={:?})",
            self.merges.as_ref().map_or(0, |m| m.len()),
            self.content_type,
            self.threads,
            self.chunk_size,
            self.memory_cap
        )
    }
}

/// Load BPE merges from a file.
///
/// # Arguments
///
/// * `path` - Path to the merges file
///
/// # Returns
///
/// Dictionary mapping (byte1, byte2) tuples to new token IDs
///
/// # Raises
///
/// * `IOError` - If file cannot be read
/// * `ValueError` - If file format is invalid
#[pyfunction]
pub fn load_bpe_merges(path: &str) -> PyResult<HashMap<(u8, u8), u16>> {
    blt_core::load_bpe_merges(&PathBuf::from(path)).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to load BPE merges: {}", e))
    })
}

/// Get the version of the BLT library.
///
/// # Returns
///
/// Version string
#[pyfunction]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// BLT (Byte-Level Tokenizer) Python bindings.
///
/// This module provides high-performance byte-level tokenization with BPE support,
/// allowing Python applications to leverage Rust-based tokenization.
#[pymodule]
fn blt(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ByteTokenizer>()?;
    m.add_function(wrap_pyfunction!(load_bpe_merges, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
