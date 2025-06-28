# Contributing to Byte-Level Tokenizer (BLT)

First off, thank you for considering contributing to BLT! Your help is greatly appreciated.

## How Can I Contribute?

### Reporting Bugs
- Ensure the bug was not already reported by searching on GitHub under [Issues](https://github.com/username/blt/issues).
- If you're unable to find an open issue addressing the problem, [open a new one](https://github.com/username/blt/issues/new). Be sure to include a title and clear description, as much relevant information as possible, and a code sample or an executable test case demonstrating the expected behavior that is not occurring.

### Suggesting Enhancements
- Open a new issue with the enhancement proposal. Clearly describe the intended feature and its benefits.

### Pull Requests
1.  Fork the repo and create your branch from `main`.
2.  If you've added code that should be tested, add tests.
3.  If you've changed APIs, update the documentation.
4.  Ensure the test suite passes (`cargo test --all`).
5.  Make sure your code lints (`cargo fmt` and `cargo clippy -- -D warnings`).
6.  Issue that pull request!

## Development Setup
1.  Clone the repository: `git clone https://github.com/username/blt.git`
2.  Install Rust: See [rustup.rs](https://rustup.rs/).
3.  Ensure you have `cargo fmt` and `cargo clippy`:
    ```bash
    rustup component add rustfmt clippy
    ```
4.  Build the project: `cargo build`
5.  Run tests: `cargo test --all`

## Coding Standards
Please review our [CODING_STANDARDS.md](./CODING_STANDARDS.md) for details on our coding style and principles.

## Code of Conduct
This project and everyone participating in it is governed by a Code of Conduct. By participating, you are expected to uphold this code. (Note: A formal Code of Conduct file should be added, e.g., Contributor Covenant).

We look forward to your contributions!
