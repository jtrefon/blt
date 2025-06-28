# Byte-Level Tokenizer (BLT)

[![Apache 2.0 License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/blt.svg)](https://crates.io/crates/blt)
[![Build Status](https://img.shields.io/github/actions/workflow/status/username/blt/ci.yml?branch=main)](https://github.com/username/blt/actions)

A high-performance, modality-agnostic byte‑level tokenizer and patching engine, designed to convert any digital content (text, audio, images, binaries) into discrete tokens for modern AI and LLM pipelines.

---

## 🚀 Features

* **Lossless Byte Coverage** – Tokenize any file as raw bytes with no unknown symbols.
* **Configurable Quantization** – Current support for Byte-Pair Encoding (BPE) merges. Entropy-based patch segmentation is a planned feature (see Roadmap v0.2).
* **Ultra‑High Throughput** – Async, multi-threaded architecture that auto-scales to available CPU cores and RAM.
* **Modular & Extensible** – Designed for modularity. Core BPE logic is in place. Pluggable strategies for different tokenizers (like patchers) and custom rules are planned for future versions to enhance extensibility.
* **Easy Integration** – Standalone CLI is available. Python bindings (via PyO3) and an optional REST adapter are planned (see Roadmap v0.3).

## 📦 Installation

**From Source (Rust CLI)**

Currently, BLT must be built from source. Publication to crates.io is planned.
```bash
git clone https://github.com/username/blt.git
cd blt
cargo build --release
# The binary will be in target/release/blt
# You can then run it as ./target/release/blt-tokenize ...
```

**Docker**

A `Dockerfile` is provided to build a Docker image locally. Official images on Docker Hub are planned.
```bash
git clone https://github.com/username/blt.git
cd blt
docker build -t blt-tokenizer .
# You can then run it as:
# docker run -i --rm blt-tokenizer --input - --output - < your_file.txt
```

**Python (future)**

```bash
pip install blt
```

---

## 🔧 Usage

### CLI

```bash
blt-tokenize \
  --input   <path/to/file>    # '-' for stdin
  --output  <path/to/output>  # '-' for stdout
  --merges  <path/to/merges.txt>   # Optional BPE merges file
  --patch   <path/to/patch.yml>    # Optional patch config
  --type    text|audio|bin    # Prepend content-type token
  --threads <num>             # Override worker count (default: detected cores)
  --memcap  <percent>         # Max RAM usage fraction (default: 80%)
  --chunksize <size>          # Min/Max chunk size (e.g. 4MB)
```

Example:

```bash
blt-tokenize -i document.pdf -o tokens.bin --type bin --merges merges.txt
```

### Python

*(Coming soon via PyO3 binding)*

```python
from blt import ByteTokenizer

tok = ByteTokenizer(merges="merges.txt", patch_config="patch.yml")
tokens = tok.encode_bytes(open("file.bin","rb").read())
```

---

## 📖 Documentation

* Architecture & design: [ARCHITECTURE.md](./ARCHITECTURE.md)
* API reference: Work in progress. Initial public API docs can be generated using `cargo doc --open`. A more formal `docs/api.md` is planned.
* Contribution guidelines: [CONTRIBUTING.md](./CONTRIBUTING.md)

---

## 🤝 Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed guidelines on how to set up your development environment, run tests, and submit pull requests.

---

## 💡 Roadmap

- **v0.1:** Core CLI, async chunked tokenization, BPE support.
- **v0.2:** Dynamic patch segmentation, advanced I/O (mmap).
- **v0.3:** Python bindings, REST microservice.
- **v1.0:** Stable release with plugin ecosystem.

---

## 📜 License

This project is licensed under the **Apache License, Version 2.0**. See [LICENSE](./LICENSE) for details.

---

## 🏷️ Acknowledgments

Inspired by:
- OpenAI GPT-2 byte-level BPE
- Google ByT5 robustness research
- Byte Latent Transformer (entropy-based patching)

Feel free to open issues or discussions for ideas, bugs, and feature requests!

```
