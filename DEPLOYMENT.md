# Deployment Guide

This document outlines the deployment process for BLT releases, including both CLI binaries and Python packages.

## 🚀 Release Process

### Prerequisites

1. **GitHub Repository Settings:**
   - Enable "Trusted Publishing" for PyPI in repository settings
   - Configure environment protection rules for `release` environment
   - Set up PyPI trusted publishing (see PyPI Setup section below)

2. **Local Development:**
   - Ensure all tests pass: `cargo test --all`
   - Ensure code is properly formatted: `cargo fmt --check`
   - Ensure no linting issues: `cargo clippy`

### Creating a Release

1. **Update Version Numbers:**
   ```bash
   # Update version in all Cargo.toml files
   # - Cargo.toml (main package)
   # - blt_core/Cargo.toml
   # - blt_python/Cargo.toml
   # - blt_python/pyproject.toml
   ```

2. **Test Everything:**
   ```bash
   # Test Rust code
   cargo test --all
   cargo clippy
   cargo build --release
   
   # Test Python bindings
   cd blt_python
   python -m venv test_env
   source test_env/bin/activate  # On Windows: test_env\Scripts\activate
   pip install maturin pytest
   maturin develop
   python -m pytest tests/ -v
   python examples/basic_usage.py
   ```

3. **Create and Push Release Tag:**
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. **Automated Release Process:**
   - GitHub Actions will automatically:
     - Build CLI binaries for all platforms
     - Build Python wheels for all platforms
     - Create GitHub release with all artifacts
     - Publish Python package to PyPI

## 🐍 PyPI Setup (One-time)

### Option 1: Trusted Publishing (Recommended)

1. **Create PyPI Account:**
   - Go to [PyPI](https://pypi.org) and create an account
   - Verify your email address

2. **Create Project on PyPI:**
   - Go to "Your projects" → "Publishing"
   - Click "Add a new pending publisher"
   - Fill in:
     - PyPI project name: `blt-tokenizer`
     - Owner: `jtrefon` (or your GitHub username)
     - Repository name: `blt`
     - Workflow name: `ci.yml`
     - Environment name: `release`

3. **Configure GitHub Repository:**
   - Go to repository Settings → Environments
   - Create environment named `release`
   - Add protection rules (optional but recommended):
     - Required reviewers
     - Wait timer
     - Deployment branches (only `main` or tags matching `v*`)

### Option 2: API Token (Alternative)

1. **Generate API Token:**
   - Go to PyPI Account Settings → API tokens
   - Create token with scope: "Entire account" or specific to `blt-tokenizer`

2. **Add to GitHub Secrets:**
   - Go to repository Settings → Secrets and variables → Actions
   - Add new secret: `PYPI_API_TOKEN` with the token value

3. **Update CI/CD (if using tokens):**
   ```yaml
   - name: Publish to PyPI
     uses: pypa/gh-action-pypi-publish@release/v1
     with:
       password: ${{ secrets.PYPI_API_TOKEN }}
       packages-dir: wheels/
   ```

## 📦 Manual Release (if needed)

### Building CLI Binaries

```bash
# Build for current platform
cargo build --release

# Cross-compilation examples (requires additional setup)
cargo install cross
cross build --target x86_64-unknown-linux-gnu --release
cross build --target x86_64-pc-windows-gnu --release
```

### Building Python Wheels

```bash
cd blt_python

# Build wheel for current platform
pip install maturin
maturin build --release

# Build for specific targets (requires Rust target installed)
rustup target add x86_64-unknown-linux-gnu
maturin build --release --target x86_64-unknown-linux-gnu

# Upload to PyPI manually
pip install twine
twine upload dist/*.whl
```

## 🔍 Testing Releases

### Testing CLI Binary

```bash
# Download from GitHub release
wget https://github.com/jtrefon/blt/releases/download/v0.2.0/blt-linux-x86_64
chmod +x blt-linux-x86_64

# Test basic functionality
echo "hello world" | ./blt-linux-x86_64 -o output.bin
```

### Testing Python Package

```bash
# Test installation from PyPI
pip install blt-tokenizer

# Test basic functionality
python -c "import blt; print(blt.version())"
```

## 🐛 Troubleshooting

### Common Issues

**PyPI Publishing Fails:**
- Check trusted publishing configuration
- Verify environment name matches CI/CD
- Ensure project name is available on PyPI

**Wheel Building Fails:**
- Check Rust toolchain installation
- Verify Python version compatibility
- Check maturin version compatibility

**Cross-compilation Issues:**
- Install required targets: `rustup target add <target>`
- Install cross-compilation tools
- Check platform-specific dependencies

### Debugging Steps

1. **Check CI/CD Logs:**
   - Go to GitHub Actions tab
   - Click on failed workflow
   - Examine logs for specific errors

2. **Test Locally:**
   ```bash
   # Replicate CI/CD steps locally
   maturin build --release
   pip install dist/*.whl
   python -c "import blt; blt.version()"
   ```

3. **Verify Configuration:**
   ```bash
   # Check pyproject.toml
   cat blt_python/pyproject.toml
   
   # Check Cargo.toml versions
   grep version */Cargo.toml
   ```

## 📋 Release Checklist

- [ ] All tests passing (`cargo test --all`)
- [ ] Python tests passing (`pytest tests/`)
- [ ] Version numbers updated in all files
- [ ] CHANGELOG.md updated
- [ ] Documentation updated
- [ ] PyPI trusted publishing configured
- [ ] GitHub environment protection configured
- [ ] Tag created and pushed
- [ ] GitHub release created with all artifacts
- [ ] PyPI package published successfully
- [ ] Installation tested: `pip install blt-tokenizer`
- [ ] Basic functionality verified

## 🔗 Useful Links

- [PyPI Trusted Publishing Guide](https://docs.pypi.org/trusted-publishers/)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Maturin Documentation](https://maturin.rs/)
- [Cross-compilation with Rust](https://rust-lang.github.io/rustup/cross-compilation.html) 