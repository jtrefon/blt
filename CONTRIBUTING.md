# Contributing to Byte-Level Tokenizer (BLT)

First off, thank you for considering contributing to BLT! Your help is greatly appreciated.

## Quick Start

```bash
# 1. Fork and clone the repository
git clone https://github.com/jtrefon/blt.git
cd blt

# 2. Set up development environment
rustup component add rustfmt clippy
cargo build

# 3. Run tests to ensure everything works
cargo test --all

# 4. Make your changes and test
cargo fmt && cargo clippy
cargo test --all

# 5. Submit a pull request
```

---

## Development Environment Setup

### Prerequisites

- **Rust 1.70+**: Install via [rustup](https://rustup.rs/)
- **Git**: For version control
- **Optional**: Docker for containerized testing

### Initial Setup

```bash
# Clone your fork
git clone https://github.com/yourusername/blt.git
cd blt

# Install required Rust components
rustup component add rustfmt clippy

# Build the project
cargo build

# Verify everything works
cargo test --all
```

### Development Tools

```bash
# Code formatting
cargo fmt

# Linting and static analysis
cargo clippy

# Security audit
cargo install cargo-audit
cargo audit

# Documentation generation
cargo doc --open

# Benchmarking
cargo bench
```

---

## Development Workflow

### 1. Creating a Feature Branch

```bash
# Create and switch to a new branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/issue-description
```

### 2. Making Changes

**Before coding:**
- Read [CODING_STANDARDS.md](./CODING_STANDARDS.md) for style guidelines
- Review [ARCHITECTURE.md](./ARCHITECTURE.md) for system design

**While coding:**
- Write tests for new functionality
- Update documentation for API changes
- Follow the existing code patterns

### 3. Testing Your Changes

```bash
# Run all tests
cargo test --all

# Run specific test suites
cargo test --lib              # Unit tests only
cargo test --test '*'         # Integration tests only

# Run tests with output
cargo test --all -- --nocapture

# Run specific test
cargo test test_name
```

### 4. Code Quality Checks

```bash
# Format code (required)
cargo fmt

# Check for common issues (required)
cargo clippy

# Security audit (recommended)
cargo audit

# Check that documentation builds
cargo doc
```

### 5. Performance Testing

```bash
# Run benchmarks
cargo bench

# Compare performance with baseline
# Benchmarks should not regress significantly
```

### 6. Submitting Changes

```bash
# Commit your changes
git add .
git commit -m "feat: add new tokenization strategy"

# Push to your fork
git push origin feature/your-feature-name

# Create a pull request on GitHub
```

---

## Testing Guidelines

### Test Structure

- **Unit tests**: Located in each module (`#[cfg(test)]` blocks)
- **Integration tests**: Located in `tests/` directory
- **Benchmarks**: Located in `benches/` directory

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_works() {
        // Arrange
        let input = "test input";
        
        // Act
        let result = your_function(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }

    #[tokio::test]
    async fn test_async_feature() {
        // Test async functionality
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Test Coverage

- All new public functions must have tests
- Critical paths should have comprehensive test coverage
- Edge cases and error conditions should be tested

---

## Documentation Guidelines

### Code Documentation

```rust
/// Brief description of the function.
///
/// More detailed explanation if needed.
///
/// # Arguments
///
/// * `param1` - Description of parameter
/// * `param2` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// Description of when this function returns an error
///
/// # Examples
///
/// ```
/// use blt_core::your_function;
/// let result = your_function("input");
/// assert_eq!(result, "expected");
/// ```
pub fn your_function(param1: &str) -> Result<String, Error> {
    // Implementation
}
```

### README Updates

When adding new features:
- Update usage examples
- Add new command-line options to the table
- Update performance benchmarks if applicable

---

## How Can I Contribute?

### 🐛 Reporting Bugs

1. **Search existing issues** to avoid duplicates
2. **Use the bug report template** when creating new issues
3. **Include relevant information**:
   - Rust version (`rustc --version`)
   - Operating system
   - Command that caused the issue
   - Expected vs actual behavior
   - Minimal reproduction case

### 💡 Suggesting Enhancements

1. **Check the roadmap** in README.md
2. **Open a feature request** with:
   - Clear description of the feature
   - Use cases and benefits
   - Proposed implementation approach
   - Breaking change considerations

### 🔧 Code Contributions

**Good first issues:**
- Documentation improvements
- Test coverage improvements
- Performance optimizations
- Bug fixes

**Advanced contributions:**
- New tokenization strategies
- Performance improvements
- API enhancements

---

## Pull Request Process

### Before Submitting

- [ ] Code follows the style guidelines ([CODING_STANDARDS.md](./CODING_STANDARDS.md))
- [ ] Self-review of code completed
- [ ] Tests added for new functionality
- [ ] All tests pass (`cargo test --all`)
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated if needed
- [ ] Benchmarks run if performance-related

### Pull Request Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Tests added/updated
- [ ] All tests pass
- [ ] Benchmarks run (if applicable)

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
```

### Review Process

1. **Automated checks** must pass (CI/CD pipeline)
2. **Code review** by maintainers
3. **Testing** on multiple platforms
4. **Merge** after approval

---

## Performance Considerations

### Benchmarking

When making performance-related changes:

```bash
# Baseline benchmark
git checkout main
cargo bench > baseline.txt

# Your changes benchmark
git checkout your-branch
cargo bench > changes.txt

# Compare results
# Performance should not regress significantly
```

### Memory Usage

- Use `cargo test --release` for memory-intensive tests
- Profile with tools like `valgrind` or `heaptrack` when needed
- Consider memory usage in algorithm design

---

## Release Process

For maintainers:

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run full test suite
4. Create release tag
5. CI/CD handles building and publishing

---

## Getting Help

- **Questions**: Open a GitHub Discussion
- **Bugs**: Create a GitHub Issue
- **Real-time chat**: (Add Discord/Slack link if available)
- **Documentation**: Check `cargo doc --open`

---

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/). By participating, you agree to uphold this code.

---

**Thank you for contributing to BLT!** 🚀

Your contributions help make this project better for everyone.
