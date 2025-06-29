#!/usr/bin/env python3
"""
BLT Python Bindings - Basic Usage Examples

This script demonstrates how to use the BLT Python bindings for
high-performance byte-level tokenization.
"""

import blt
import tempfile
import os


def example_basic_usage():
    """Demonstrate basic file tokenization."""
    print("=== File Processing Example ===")
    
    # Create a tokenizer
    tokenizer = blt.ByteTokenizer()
    print(f"Created tokenizer: {tokenizer}")
    
    # Create a temporary input file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as input_file:
        input_file.write("hello world this is a test")
        input_path = input_file.name
    
    # Create a temporary output file
    with tempfile.NamedTemporaryFile(suffix='.bin', delete=False) as output_file:
        output_path = output_file.name
    
    try:
        print(f"Processing file: {input_path}")
        
        # Get input file size
        input_size = os.path.getsize(input_path)
        print(f"Input file size: {input_size} bytes")
        
        # Tokenize the file
        tokenizer.tokenize_file(input_path, output_path)
        
        # Check output
        output_size = os.path.getsize(output_path)
        print(f"Output file size: {output_size} bytes")
        
        # Read first 20 bytes of output
        with open(output_path, 'rb') as f:
            first_bytes = f.read(20)
        print(f"First 20 bytes of output: {first_bytes}")
        
    finally:
        # Clean up
        os.unlink(input_path)
        os.unlink(output_path)


def example_bpe_merges():
    """Demonstrate BPE tokenization with custom merges."""
    print("\n=== BPE Merges Example ===")
    
    # Create a temporary merges file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as merges_file:
        merges_file.write("97 98\n")   # 'a' + 'b' -> token 256
        merges_file.write("99 100\n")  # 'c' + 'd' -> token 257
        merges_file.write("101 102\n") # 'e' + 'f' -> token 258
        merges_path = merges_file.name
    
    try:
        print(f"Loading merges from: {merges_path}")
        
        # Load merges from file
        merges = blt.load_bpe_merges(merges_path)
        print(f"Loaded merges: {merges}")
        
        # Create tokenizer with BPE merges
        tokenizer = blt.ByteTokenizer(merges=merges)
        print(f"Created BPE tokenizer: {tokenizer}")
        
        # Create input file with mergeable content
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as input_file:
            input_file.write("abcdef")
            input_path = input_file.name
        
        # Create output file
        with tempfile.NamedTemporaryFile(suffix='.bin', delete=False) as output_file:
            output_path = output_file.name
        
        try:
            print("Processing text: 'abcdef'")
            
            # Tokenize with BPE
            tokenizer.tokenize_file(input_path, output_path)
            
            # Read the result
            with open(output_path, 'rb') as f:
                result = f.read()
            
            print(f"Original: abcdef ({len('abcdef')} chars)")
            print(f"Tokenized: {result} ({len(result)} bytes)")
            
        finally:
            os.unlink(input_path)
            os.unlink(output_path)
            
    finally:
        os.unlink(merges_path)


def example_configuration_options():
    """Demonstrate various configuration options."""
    print("\n=== Configuration Example ===")
    
    # Different configuration examples
    configs = [
        {},  # Default
        {"content_type": "Text"},
        {"threads": 2},
        {"chunk_size": "1MB"},
        {"memory_cap": 50},
        {"content_type": "Bin", "threads": 4, "memory_cap": 90},
    ]
    
    for i, config in enumerate(configs, 1):
        tokenizer = blt.ByteTokenizer(**config)
        print(f"Config {i}: {tokenizer}")


def main():
    """Run all examples."""
    print(f"BLT version: {blt.version()}")
    print(f"Module version: {blt.__version__}")
    print("BLT Python Bindings Example")
    print("=" * 40)
    
    try:
        example_basic_usage()
        example_bpe_merges()
        example_configuration_options()
        
        print("\n=== Example completed successfully! ===")
        
    except Exception as e:
        print(f"\nError: {e}")
        raise


if __name__ == "__main__":
    main() 