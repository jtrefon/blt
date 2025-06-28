# Byte-Level Tokenizer Architecture

This document details the high-performance architecture for the **Byte-Level Tokenizer** (BLT) open-source project. We focus on maximizing throughput and resource utilization via multi-threading, async processing, and robust job routing.

---

## 1. Introduction

The Byte-Level Tokenizer (BLT) converts arbitrary digital content (text, audio, images, binaries) into discrete tokens at the byte or composite patch level. Our goal is a world-class, bit-level tokenizer that fully leverages modern multi-core hardware and memory architectures.

### 1.1 Motivation

* **Ultra-High Throughput:** Tokenize millions of bytes per second.
* **Scalability:** Automatically adjust to available CPU cores and RAM.
* **Reliability:** Async, non-blocking pipelines with fault isolation.
* **Extensibility:** Modular design for custom quantization or patch strategies.

## 2. High-Level System Overview

```text
+------------------+       +------------------+      +------------------+
|  CLI / REST API  |--(1)->|  Job Router      |--+-->| Worker Pool      |
+------------------+       +------------------+  |   +------------------+
                                                 |
                                                 +-->+ Memory Manager   |
                                                     +------------------+
                                                          |
                                                          v
                                               +----------------------+ 
                                               | Chunk Dispatcher     | 
                                               +----------------------+ 
                                                          |
                                                          v
                                               +----------------------+ 
                                               | Tokenization Engine  | 
                                               +----------------------+ 
                                                          |
                                                          v
                                               +----------------------+ 
                                               | Shard Assembler      | 
                                               +----------------------+ 
                                                          |
                                                          v
                                               +----------------------+ 
                                               | Output Serializer    | 
                                               +----------------------+ 
```

1. **CLI / REST API:** Single-input single-output entrypoint.
2. **Job Router:** Async frontdoor that schedules tokenization jobs.
3. **Worker Pool:** Thread- or coroutine-based pool sized to CPU cores.
4. **Memory Manager:** Monitors and caps RAM usage for chunk buffers.
5. **Chunk Dispatcher:** Breaks input into shards/chunks for parallel work.
6. **Tokenization Engine:** Performs byte-reading, BPE/patch logic per chunk.
7. **Shard Assembler:** Reorders and merges chunk outputs.
8. **Output Serializer:** Writes final token ID stream to stdout/file.

## 3. Resource Detection & Initialization

* **Core Detection:** At startup, detect `N = number_of_logical_processors()`.
* **RAM Check:** Query total and available system RAM. Reserve a configurable fraction (e.g. 80%) for token buffers.
* **Threading Model:** Create a worker pool with `min(N, max_workers)` threads or async tasks.
* **Async Runtime:** Use Tokio (Rust) or equivalent async engine to schedule tasks non-blockingly.

## 4. Input Chunking & Sharding

* **Chunk Size Determination:** Compute chunk size = `clamp(total_bytes / N, min_chunk, max_chunk)` to balance load.
* **Alignment:** Ensure chunks align to safe boundaries (e.g. avoid splitting multi-byte patterns mid-stream).
* **Dispatch:** `Memory Manager` allocates buffer for each chunk; `Chunk Dispatcher` enqueues chunk tasks to `Worker Pool`.

## 5. Tokenization Engine (Worker Tasks)

Each worker proceeds asynchronously:

1. **Read Bytes:** Stream chunk into fast in-memory `uint8` slice.
2. **Preprocessing:** If enabled, strip or normalize headers (e.g. remove BOM).
3. **Quantization:** Apply BPE merges or entropy-based patch rules in-place.
4. **Meta-Token Injection:** Prepend content-type token if specified.
5. **Token ID Emission:** Write local token IDs into a per-chunk buffer.
6. **Completion Signal:** Notify `Shard Assembler` of chunk readiness.

## 6. Shard Assembly & Ordering

* **Future-Promise Pattern:** Workers return `(chunk_index, token_buffer)` futures.
* **Ordered Collector:** `Shard Assembler` awaits futures in index order, concatenating buffers.
* **Backpressure:** If downstream is slow, throttle chunk dispatch to prevent memory overload.

## 7. Memory Management

* **Dynamic Buffer Pool:** Pre-allocate a pool of `M` chunk buffers sized by `max_chunk`. Reuse buffers to reduce GC/alloc overhead.
* **RAM Cap Enforcement:** Track total buffer memory; if usage exceeds threshold, pause dispatch until buffers free.
* **Zero-Copy I/O:** Use memory-mapped files or vectorized I/O for large inputs where supported.

## 8. Concurrency & Async Patterns

* **Job Router:** Implements an async queue with configurable `max_inflight_jobs`.
* **Worker Pool:** Uses a work-stealing scheduler for balanced CPU usage.
* **Backpressure Channels:** Bounded channels between modules ensure non-blocking flow control.

## 9. CLI & Configurable Flags

```bash
blt-tokenize \
  --input   file.bin       # Path or '-' for stdin
  --output  tokens.bin      # Path or '-' for stdout
  --merges  merges.txt      # Optional BPE merge rules
  --patch   patch.yml       # Optional patch config
  --type    text|audio|bin|video  # Content-type token
  --threads 8               # Override worker count
  --memcap  80%             # Max RAM usage fraction
  --chunksize 4MB           # Min/Max chunk size bounds
```

## 10. Extension Points

* **MergeStrategy Trait:** Implement custom BPE or patch algorithms.
* **ChunkBalancer:** Custom logic to adapt chunk sizes based on real-time throughput.
* **Output Formats:** Plug in Protobuf, Avro, or JSON serializers.
* **REST Adapter:** Separate crate to wrap CLI behind HTTP endpoints.

## 11. Testing & Benchmarking

* **Unit Tests:** Validate tokenization correctness on known inputs.
* **Integration Tests:** End-to-end CLI with sample binaries.
* **Performance Benchmarks:** Measure MB/s throughput across varying `threads` and `chunksize` on representative hardware.

## 12. Roadmap

* **Phase 1:** Core CLI, async chunked tokenization, BPE support.
* **Phase 2:** Dynamic patch segmentation, advanced memory-mapped I/O.
* **Phase 3:** Python binding (PyO3), REST wrapper, cloud-deployed microservice.

