"""
Tests for BLT Python bindings.
"""

import pytest
import tempfile
import os
import blt


class TestByteTokenizer:
    """Test cases for ByteTokenizer class."""

    def test_basic_tokenizer_creation(self):
        """Test creating a basic tokenizer."""
        tokenizer = blt.ByteTokenizer()
        assert tokenizer is not None
        assert "ByteTokenizer" in str(tokenizer)

    def test_tokenizer_with_merges(self):
        """Test creating a tokenizer with BPE merges."""
        merges = {(97, 98): 256, (99, 100): 257}
        tokenizer = blt.ByteTokenizer(merges=merges)
        assert "merges=2" in str(tokenizer)

    def test_tokenizer_with_content_type(self):
        """Test creating a tokenizer with content type."""
        tokenizer = blt.ByteTokenizer(content_type="Text")
        assert 'content_type=Some("Text")' in str(tokenizer)
        
        tokenizer = blt.ByteTokenizer(content_type="Bin")
        assert 'content_type=Some("Bin")' in str(tokenizer)

    def test_invalid_content_type(self):
        """Test that invalid content types raise ValueError."""
        with pytest.raises(ValueError):
            blt.ByteTokenizer(content_type="Invalid")

    def test_invalid_memory_cap(self):
        """Test that invalid memory cap values raise ValueError."""
        with pytest.raises(ValueError):
            blt.ByteTokenizer(memory_cap=150)  # Over 100%
        
        # Note: negative values are not currently validated in the Rust code

    def test_basic_tokenization(self):
        """Test basic tokenization functionality."""
        tokenizer = blt.ByteTokenizer()
        
        # Create temporary input file
        with tempfile.NamedTemporaryFile(mode='wb', delete=False) as input_file:
            input_file.write(b"hello world")
            input_path = input_file.name
        
        # Create temporary output file
        with tempfile.NamedTemporaryFile(delete=False) as output_file:
            output_path = output_file.name
        
        try:
            tokenizer.tokenize_file(input_path, output_path)
            
            # Check that output file was created and has content
            assert os.path.exists(output_path)
            with open(output_path, 'rb') as f:
                result = f.read()
            assert len(result) > 0
            
        finally:
            # Clean up
            os.unlink(input_path)
            os.unlink(output_path)

    def test_empty_input(self):
        """Test tokenization with empty input."""
        tokenizer = blt.ByteTokenizer()
        
        # Create temporary input file
        with tempfile.NamedTemporaryFile(mode='wb', delete=False) as input_file:
            input_file.write(b"")
            input_path = input_file.name
        
        # Create temporary output file
        with tempfile.NamedTemporaryFile(delete=False) as output_file:
            output_path = output_file.name
        
        try:
            tokenizer.tokenize_file(input_path, output_path)
            
            # Check that output file was created
            assert os.path.exists(output_path)
            with open(output_path, 'rb') as f:
                result = f.read()
            assert isinstance(result, bytes)
            
        finally:
            # Clean up
            os.unlink(input_path)
            os.unlink(output_path)

    def test_bpe_tokenization(self):
        """Test BPE tokenization with merges."""
        # Simple merge: 'ab' -> token 256
        merges = {(97, 98): 256}  # 'a' + 'b' -> token 256
        tokenizer = blt.ByteTokenizer(merges=merges)
        
        # Create temporary input file
        with tempfile.NamedTemporaryFile(mode='wb', delete=False) as input_file:
            input_file.write(b"ab")
            input_path = input_file.name
        
        # Create temporary output file
        with tempfile.NamedTemporaryFile(delete=False) as output_file:
            output_path = output_file.name
        
        try:
            tokenizer.tokenize_file(input_path, output_path)
            
            # Check that output file was created and has content
            assert os.path.exists(output_path)
            with open(output_path, 'rb') as f:
                result = f.read()
            
            assert isinstance(result, bytes)
            assert len(result) > 0
            
        finally:
            # Clean up
            os.unlink(input_path)
            os.unlink(output_path)

    def test_file_tokenization(self):
        """Test file-based tokenization."""
        tokenizer = blt.ByteTokenizer()
        
        # Create test input file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as input_file:
            input_file.write("This is a test file for tokenization.")
            input_path = input_file.name
        
        # Create output file path
        with tempfile.NamedTemporaryFile(suffix='.bin', delete=False) as output_file:
            output_path = output_file.name
        
        try:
            # Tokenize the file
            tokenizer.tokenize_file(input_path, output_path)
            
            # Verify output exists and has content
            assert os.path.exists(output_path)
            assert os.path.getsize(output_path) > 0
            
        finally:
            # Clean up
            os.unlink(input_path)
            os.unlink(output_path)

    def test_configuration_options(self):
        """Test various configuration options."""
        tokenizer = blt.ByteTokenizer(
            threads=2,
            chunk_size="1MB",
            memory_cap=50
        )
        
        # Create temporary input file
        with tempfile.NamedTemporaryFile(mode='wb', delete=False) as input_file:
            input_file.write(b"test data for configuration")
            input_path = input_file.name
        
        # Create temporary output file
        with tempfile.NamedTemporaryFile(delete=False) as output_file:
            output_path = output_file.name
        
        try:
            tokenizer.tokenize_file(input_path, output_path)
            
            # Check that output file was created and has content
            assert os.path.exists(output_path)
            with open(output_path, 'rb') as f:
                result = f.read()
            assert isinstance(result, bytes)
            
        finally:
            # Clean up
            os.unlink(input_path)
            os.unlink(output_path)

    def test_large_data(self):
        """Test tokenization with larger data."""
        tokenizer = blt.ByteTokenizer()
        
        # Create temporary input file with 100KB of test data
        with tempfile.NamedTemporaryFile(mode='wb', delete=False) as input_file:
            large_data = b"x" * (100 * 1024)  # 100KB
            input_file.write(large_data)
            input_path = input_file.name
        
        # Create temporary output file
        with tempfile.NamedTemporaryFile(delete=False) as output_file:
            output_path = output_file.name
        
        try:
            tokenizer.tokenize_file(input_path, output_path)
            
            # Check that output file was created and has content
            assert os.path.exists(output_path)
            with open(output_path, 'rb') as f:
                result = f.read()
            
            assert isinstance(result, bytes)
            assert len(result) > 0
            
        finally:
            # Clean up
            os.unlink(input_path)
            os.unlink(output_path)


class TestUtilityFunctions:
    """Test cases for utility functions."""

    def test_version_function(self):
        """Test the version function."""
        version = blt.version()
        assert isinstance(version, str)
        assert len(version) > 0
        assert "." in version  # Should be semantic version

    def test_load_bpe_merges_file_not_found(self):
        """Test loading BPE merges from non-existent file."""
        with pytest.raises(IOError):
            blt.load_bpe_merges("non_existent_file.txt")

    def test_load_bpe_merges_valid_file(self):
        """Test loading BPE merges from valid file."""
        # Create temporary merges file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False) as merges_file:
            merges_file.write("97 98\n")  # a b
            merges_file.write("99 100\n")  # c d
            merges_path = merges_file.name
        
        try:
            merges = blt.load_bpe_merges(merges_path)
            assert isinstance(merges, dict)
            assert len(merges) == 2
            assert (97, 98) in merges
            assert (99, 100) in merges
            
        finally:
            os.unlink(merges_path)


class TestModuleAttributes:
    """Test module-level attributes and exports."""

    def test_module_version(self):
        """Test module version attribute."""
        assert hasattr(blt, '__version__')
        assert isinstance(blt.__version__, str)
        assert blt.__version__ == blt.version()

    def test_module_exports(self):
        """Test that all expected symbols are exported."""
        expected_exports = ['ByteTokenizer', 'load_bpe_merges', 'version', '__version__']
        for export in expected_exports:
            assert hasattr(blt, export), f"Missing export: {export}"


class TestPerformance:
    """Performance-related tests."""

    def test_performance_benchmark(self):
        """Basic performance test."""
        import time
        
        tokenizer = blt.ByteTokenizer()
        
        # Create temporary input file with 100KB of test data
        with tempfile.NamedTemporaryFile(mode='wb', delete=False) as input_file:
            data = b"x" * (100 * 1024)  # 100KB test data
            input_file.write(data)
            input_path = input_file.name
        
        # Create temporary output file
        with tempfile.NamedTemporaryFile(delete=False) as output_file:
            output_path = output_file.name
        
        try:
            start_time = time.time()
            tokenizer.tokenize_file(input_path, output_path)
            end_time = time.time()
            
            duration = end_time - start_time
            
            # Should complete quickly (less than 1 second for 100KB)
            assert duration < 1.0
            
            # Check results
            with open(output_path, 'rb') as f:
                result = f.read()
            assert isinstance(result, bytes)
            assert len(result) > 0
            
        finally:
            # Clean up
            os.unlink(input_path)
            os.unlink(output_path) 