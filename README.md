# Byte-Level Tokenizer (BLT)

[![Apache 2.0 License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/blt.svg)](https://crates.io/crates/blt)
[![Build Status](https://img.shields.io/github/actions/workflow/status/jtrefon/blt/ci.yml?branch=main)](https://github.com/jtrefon/blt/actions)

A high-performance, modality-agnostic byte‑level tokenizer designed to convert any digital content (text, audio, images, binaries) into discrete tokens for modern AI and LLM pipelines.

**Performance**: Processes 100MB files in ~38ms with memory-mapped I/O and concurrent processing.

---

## 🚀 Features

* **Lossless Byte Coverage** – Tokenize any file as raw bytes with no unknown symbols.
* **High Performance** – Memory-mapped I/O with async, multi-threaded processing pipeline.
* **Configurable Strategies** – Support for Byte-Pair Encoding (BPE) merges and passthrough tokenization.
* **Auto-scaling** – Automatically detects and utilizes available CPU cores and RAM.
* **Flexible I/O** – Supports files, stdin/stdout, with configurable chunk sizing and memory limits.
* **Production Ready** – Comprehensive testing, benchmarking, CI/CD, and structured logging.

---

## 📦 Installation & Building

### Prerequisites

- **Rust** 1.70+ (install via [rustup](https://rustup.rs/))
- **Git** for cloning the repository

### From Source

```bash
# Clone the repository
git clone https://github.com/jtrefon/blt.git
cd blt

# Build in release mode for optimal performance
cargo build --release

# The binary will be available at:
# ./target/release/blt
```

### Development Build

```bash
# Build in debug mode (faster compilation, slower runtime)
cargo build

# The debug binary will be available at:
# ./target/debug/blt
```

### Docker

```bash
# Build Docker image
docker build -t blt-tokenizer .

# Run with Docker
echo "Hello, world!" | docker run -i --rm blt-tokenizer --input - --output -
```

### Pre-built Binaries

Release binaries are automatically built for multiple platforms via CI/CD:

- **Linux (x86_64)**: `blt-linux-amd64.tar.gz`
- **macOS Intel (x86_64)**: `blt-macos-amd64.tar.gz`
- **macOS Apple Silicon (ARM64)**: `blt-macos-arm64.tar.gz`
- **Windows (x86_64)**: `blt-windows-amd64.zip`

Download the latest release from [GitHub Releases](https://github.com/jtrefon/blt/releases).

---

## 🔧 Usage

### Command Line Interface

```bash
blt [OPTIONS]
```

#### Options

| Option | Description | Default |
|--------|-------------|---------|
| `-i, --input <PATH>` | Input file path (use `-` for stdin) | stdin |
| `-o, --output <PATH>` | Output file path (use `-` for stdout) | stdout |
| `-m, --merges <PATH>` | BPE merges file (optional) | None (passthrough mode) |
| `-t, --type <TYPE>` | Content type: `text`, `audio`, `bin`, `video` | None |
| `--threads <NUM>` | Number of processing threads | Auto-detected CPU cores |
| `--chunksize <SIZE>` | Chunk size (e.g., `16MB`, `1024KB`) | Auto-calculated |
| `--memcap <PERCENT>` | Max RAM usage percentage | 80% |
| `-h, --help` | Show help information | |
| `-V, --version` | Show version information | |

#### Examples

**Basic Usage:**
```bash
# Tokenize a text file
./target/release/blt -i document.txt -o tokens.bin

# Use stdin/stdout
echo "Hello, world!" | ./target/release/blt --input - --output -

# Specify content type
./target/release/blt -i video.mp4 -o tokens.bin --type video
```

**With BPE Merges:**
```bash
# Apply BPE tokenization
./target/release/blt -i input.txt -o output.bin --merges merges.txt

# Example merges.txt format:
# 97 98    # 'a' + 'b' -> new token 256
# 99 100   # 'c' + 'd' -> new token 257
```

**Performance Tuning:**
```bash
# Use 8 threads with 2MB chunks
./target/release/blt -i large_file.bin -o output.bin --threads 8 --chunksize 2MB

# Limit memory usage to 50%
./target/release/blt -i huge_file.bin -o output.bin --memcap 50
```

---

## 🧪 Development & Testing

### Running Tests

```bash
# Run all tests (unit + integration)
cargo test --all

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run tests with output
cargo test --all -- --nocapture

# Run specific test
cargo test test_bpe_strategy_simple_merge
```

### Benchmarking

```bash
# Run performance benchmarks
cargo bench

# View benchmark results
# Results are saved to target/criterion/

# Run specific benchmark
cargo bench passthrough_100mb_file
```

**Expected Performance:**
- **100MB file processing**: ~38ms
- **Memory usage**: Scales with available RAM and configured limits
- **Throughput**: Optimized for both small and large files

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Security audit
cargo audit

# Generate documentation
cargo doc --open
```

### Development Workflow

```bash
# 1. Make changes to code
# 2. Format and check
cargo fmt && cargo clippy

# 3. Run tests
cargo test --all

# 4. Run benchmarks (optional)
cargo bench

# 5. Build release
cargo build --release
```

---

## 🏗️ Architecture

The project follows a modular architecture with clear separation of concerns:

- **`blt_core`**: Core library with tokenization logic
- **`src/main.rs`**: CLI interface and argument parsing
- **`benches/`**: Performance benchmarks
- **`tests/`**: Integration tests

### Key Components

- **Pipeline**: Async multi-threaded processing engine
- **Strategies**: Pluggable tokenization algorithms (BPE, Passthrough)
- **I/O Handler**: Memory-mapped files and streaming I/O
- **Chunking**: Dynamic chunk sizing based on system resources

For detailed architecture information, see [ARCHITECTURE.md](./ARCHITECTURE.md).

---

## 📊 Performance

### Benchmarks

Current performance on a typical development machine:

| File Size | Processing Time | Throughput |
|-----------|----------------|------------|
| 1MB | ~0.4ms | ~2.5 GB/s |
| 10MB | ~3.8ms | ~2.6 GB/s |
| 100MB | ~38ms | ~2.6 GB/s |

### Optimization Features

- **Memory-mapped I/O**: Zero-copy file processing
- **Concurrent Processing**: Multi-threaded chunk processing
- **Dynamic Scaling**: Automatic resource detection
- **Efficient Algorithms**: Optimized BPE implementation

---

## 🐛 Troubleshooting

### Common Issues

**Out of Memory:**
```bash
# Reduce memory cap or chunk size
./target/release/blt -i large_file.bin -o output.bin --memcap 50 --chunksize 1MB
```

**Slow Performance:**
```bash
# Increase thread count or chunk size
./target/release/blt -i file.bin -o output.bin --threads 16 --chunksize 16MB
```

**Build Errors:**
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean && cargo build --release
```

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug ./target/release/blt -i input.txt -o output.bin

# Enable trace logging for detailed output
RUST_LOG=trace ./target/release/blt -i input.txt -o output.bin
```

---

## 📖 Documentation

* **Architecture & Design**: [ARCHITECTURE.md](./ARCHITECTURE.md)
* **API Reference**: Run `cargo doc --open` for detailed API documentation
* **Contributing**: [CONTRIBUTING.md](./CONTRIBUTING.md)
* **Coding Standards**: [CODING_STANDARDS.md](./CODING_STANDARDS.md)

---

## 🤝 Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed guidelines.

### Quick Start for Contributors

```bash
# 1. Fork and clone
git clone https://github.com/jtrefon/blt.git
cd blt

# 2. Create a feature branch
git checkout -b feature/your-feature-name

# 3. Make changes and test
cargo test --all
cargo clippy
cargo fmt

# 4. Commit and push
git commit -m "Add your feature"
git push origin feature/your-feature-name

# 5. Create a pull request
```

---

## 💡 Roadmap

- **v0.1** ✅ Core CLI, async chunked tokenization, BPE support
- **v0.2** ✅ Memory-mapped I/O, performance optimization, comprehensive testing
- **v0.3** 🚧 Python bindings, REST microservice
- **v1.0** 📋 Stable release with plugin ecosystem

---

## 📜 License

This project is licensed under the **Apache License, Version 2.0**. See [LICENSE](./LICENSE) for details.

---

## 🏷️ Acknowledgments

Inspired by:
- OpenAI GPT-2 byte-level BPE
- Google ByT5 robustness research
- Byte Latent Transformer (entropy-based patching)

---

**Questions?** Open an issue or start a discussion. We're here to help! 🚀

```
