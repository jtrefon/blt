# Byte-Level Tokenizer (BLT)

[![Apache 2.0 License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/blt.svg)](https://crates.io/crates/blt)
[![Build Status](https://img.shields.io/github/actions/workflow/status/username/blt/ci.yml?branch=main)](https://github.com/username/blt/actions)

A high-performance, modality-agnostic byte‑level tokenizer and patching engine, designed to convert any digital content (text, audio, images, binaries) into discrete tokens for modern AI and LLM pipelines.

---

## 🚀 Features

* **Lossless Byte Coverage** – Tokenize any file as raw bytes with no unknown symbols.
* **Configurable Quantization** – Support for Byte-Pair Encoding (BPE) merges and entropy-based patch segmentation.
* **Ultra‑High Throughput** – Async, multi-threaded architecture that auto-scales to available CPU cores and RAM.
* **Modular & Extensible** – Pluggable strategies for BPE, patchers, and custom tokenization rules.
* **Easy Integration** – Standalone CLI, Python bindings (via PyO3), and optional REST adapter.

## 📦 Installation

**Rust (CLI only)**

```bash
cargo install blt
```

**Docker**

```bash
docker pull username/blt:latest
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

* Architecture & design: [Architecture.md](./Architecture.md)
* API reference (once published): [docs/api.md](./docs/api.md)

---

## 🤝 Contributing

We welcome contributions of all kinds:

1. **Clone** the repo and create a feature branch:

   ```bash
   ```

git clone [https://github.com/username/blt.git](https://github.com/username/blt.git)
cd blt
git checkout -b feature/your-idea

```
2. **Implement** your changes, with clear tests under `tests/`.
3. **Format** code (`cargo fmt`) and **lint** (`cargo clippy`).
4. **Push** and open a Pull Request targeting `main`.

Please review our [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed guidelines.

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
