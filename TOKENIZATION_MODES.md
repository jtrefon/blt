# BLT Tokenization Modes

This document explains the different tokenization modes available in BLT and clarifies the behavior changes made to improve user experience.

## 🔄 What Changed

Previously, BLT had confusing default behavior:
- **Without BPE merges**: Files were copied unchanged (passthrough mode)
- **With BPE merges**: Files were actually tokenized

This caused confusion because:
1. The tool is called a "**Tokenizer**" but didn't tokenize by default
2. Python examples showed basic usage without merges, implying tokenization
3. Users expected tokenization but got file copying

## ✅ New Behavior (Fixed)

BLT now has three clear modes:

### 1. Basic Tokenization (New Default)
**When**: No `--merges` file provided and no `--passthrough` flag
**Behavior**: Each byte → 16-bit token (big-endian)
**File size**: ~2x original size
**Use case**: Simple tokenization for AI/LLM workflows

```bash
# CLI
./blt -i input.txt -o tokens.bin

# Python
tokenizer = blt.ByteTokenizer()
tokenizer.tokenize_file("input.txt", "tokens.bin")
```

### 2. Advanced BPE Tokenization
**When**: `--merges` file is provided
**Behavior**: Basic tokenization + byte-pair merging for compression
**File size**: Usually smaller than basic tokenization
**Use case**: Optimized tokenization with compression

```bash
# CLI
./blt -i input.txt -o tokens.bin --merges merges.txt

# Python
merges = blt.load_bpe_merges("merges.txt")
tokenizer = blt.ByteTokenizer(merges=merges)
tokenizer.tokenize_file("input.txt", "tokens.bin")
```

### 3. Passthrough Mode (Explicit File Copying)
**When**: `--passthrough` flag is used
**Behavior**: File copied unchanged
**File size**: Identical to original
**Use case**: File copying without tokenization

```bash
# CLI only (Python API doesn't support passthrough)
./blt -i input.txt -o copy.txt --passthrough
```

## 📊 Output Format Comparison

| Mode | Input: "hello" (5 bytes) | Output Size | Output Content |
|------|-------------------------|-------------|----------------|
| **Basic** | `[104, 101, 108, 108, 111]` | 10 bytes | `[0, 104, 0, 101, 0, 108, 0, 108, 0, 111]` |
| **BPE** | `[104, 101, 108, 108, 111]` | Varies | Depends on merges |
| **Passthrough** | `[104, 101, 108, 108, 111]` | 5 bytes | `[104, 101, 108, 108, 111]` |

## 🎯 For Your Video File

Your 1.1GB video file was processed in **passthrough mode** (the old default), which just copied it.

To actually tokenize it:

```bash
# Basic tokenization (will create ~2.2GB output)
./blt -i video.mkv -o video_tokens.bin --type video

# With BPE merges (will vary based on merges)
./blt -i video.mkv -o video_tokens.bin --merges video_merges.txt --type video
```

## 🧪 Testing the Changes

```bash
# Test basic tokenization
echo "test" | ./blt | xxd
# Output: 00000000: 0074 0065 0073 0074 000a  .t.e.s.t..

# Test passthrough mode  
echo "test" | ./blt --passthrough
# Output: test
```

## 🔧 Migration Guide

### If you were relying on the old passthrough behavior:
- Add `--passthrough` flag to CLI commands
- Python API never supported passthrough, so no changes needed

### If you expected tokenization but got file copying:
- No changes needed! It now works as expected by default

## 📚 Documentation Updates

All documentation has been updated to reflect the new behavior:
- ✅ Main README.md
- ✅ Python README.md  
- ✅ CLI help text
- ✅ Python examples
- ✅ API documentation
- ✅ Tests updated

The behavior is now consistent across CLI, Python API, and documentation. 