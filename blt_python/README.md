# BLT Python Bindings

High-performance Python bindings for BLT (Byte-Level Tokenizer), providing fast byte-level tokenization with BPE support.

## 🚀 Features

- **High Performance**: Rust-powered tokenization with multi-threading support
- **BPE Support**: Byte-Pair Encoding with custom merge rules
- **Memory Efficient**: Memory-mapped I/O and configurable memory usage
- **Easy to Use**: Simple Python API with comprehensive error handling
- **Cross-Platform**: Works on Linux, macOS, and Windows

## 📦 Installation

```bash
pip install blt-tokenizer
```

### Development Installation

```bash
# Clone the repository
git clone https://github.com/jtrefon/blt.git
cd blt/blt_python

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install maturin and build
# On Linux
pip install maturin[patchelf]

# On macOS or Windows
pip install maturin

# Build and install for development
maturin develop

# Install development dependencies
pip install -e .[dev]
```

## 🔧 Usage

### Basic File Tokenization

```python
import blt

# Create a tokenizer
tokenizer = blt.ByteTokenizer()

# Tokenize a file
tokenizer.tokenize_file("input.txt", "output.bin")
```

### BPE Tokenization

```python
import blt

# Load BPE merges from file
merges = blt.load_bpe_merges("merges.txt")

# Or define merges manually
merges = {
    (97, 98): 256,   # 'a' + 'b' -> token 256
    (99, 100): 257,  # 'c' + 'd' -> token 257
    (101, 102): 258, # 'e' + 'f' -> token 258
}

# Create tokenizer with BPE merges
tokenizer = blt.ByteTokenizer(merges=merges)
tokenizer.tokenize_file("input.txt", "output.bin")
```

### Advanced Configuration

```python
import blt

# Create tokenizer with custom settings
tokenizer = blt.ByteTokenizer(
    content_type="Text",    # "Text" or "Bin"
    threads=4,              # Number of processing threads
    chunk_size="1MB",       # Chunk size for processing
    memory_cap=50           # Memory usage cap (0-100%)
)

tokenizer.tokenize_file("large_file.txt", "output.bin")
```

## 📖 API Reference

### `ByteTokenizer`

Main tokenizer class for byte-level tokenization.

#### Constructor

```python
ByteTokenizer(
    merges=None,        # Dict[Tuple[int, int], int] - BPE merge rules
    content_type=None,  # str - "Text" or "Bin"
    threads=None,       # int - Number of threads
    chunk_size=None,    # str - Chunk size (e.g., "1MB")
    memory_cap=None     # int - Memory cap percentage (0-100)
)
```

#### Methods

- **`tokenize_file(input_path, output_path)`**: Tokenize a file and save results
  - `input_path` (str): Path to input file
  - `output_path` (str): Path to output file
  - Raises: `RuntimeError`, `IOError`

### Utility Functions

- **`load_bpe_merges(path)`**: Load BPE merges from file
  - Returns: `Dict[Tuple[int, int], int]`
  - Raises: `IOError`, `ValueError`

- **`version()`**: Get library version
  - Returns: `str`

## 🧪 Testing

```bash
# Run tests
python -m pytest tests/

# Run with coverage
python -m pytest tests/ --cov=blt

# Run benchmarks
python -m pytest tests/ -k benchmark
```

## ⚡ Performance

The Python bindings maintain the same high performance as the CLI version:

| File Size | Processing Time | Throughput |
|-----------|----------------|------------|
| 1MB | ~0.4ms | ~2.5 GB/s |
| 10MB | ~3.8ms | ~2.6 GB/s |
| 100MB | ~38ms | ~2.6 GB/s |

## 🔍 Examples

See the `examples/` directory for comprehensive usage examples:

- `basic_usage.py` - Basic tokenization examples
- `bpe_example.py` - BPE tokenization with custom merges
- `performance_test.py` - Performance benchmarking

## 🐛 Troubleshooting

### Common Issues

**ImportError: No module named 'blt'**
```bash
# Ensure package is installed
pip install blt-tokenizer

# Or for development
maturin develop
```

**RuntimeError during tokenization**
```bash
# Check file permissions and paths
# Reduce memory usage if needed
tokenizer = blt.ByteTokenizer(memory_cap=50)
```

**Performance issues**
```bash
# Increase thread count and chunk size
tokenizer = blt.ByteTokenizer(threads=8, chunk_size="16MB")
```

## 📄 License

This project is licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## 🤝 Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## 🔗 Links

- [Main Repository](https://github.com/jtrefon/blt)
- [Documentation](https://github.com/jtrefon/blt#readme)
- [Issue Tracker](https://github.com/jtrefon/blt/issues)
- [PyPI Package](https://pypi.org/project/blt-tokenizer/) 