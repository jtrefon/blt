"""
BLT (Byte-Level Tokenizer) Python Bindings

High-performance byte-level tokenization with BPE support, powered by Rust.

Example usage:
    >>> import blt
    >>> tokenizer = blt.ByteTokenizer()
    >>> tokenizer.tokenize_file("input.txt", "output.bin")
"""

from .blt import ByteTokenizer, load_bpe_merges, version

__version__ = version()

__all__ = ["ByteTokenizer", "load_bpe_merges", "version", "__version__"] 