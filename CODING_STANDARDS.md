# Coding Standards for Byte-Level Tokenizer (BLT) Project

This document outlines the coding standards and principles to be followed by all contributors to the Byte-Level Tokenizer (BLT) project. Adhering to these standards will help maintain code quality, readability, and long-term maintainability.

## 1. Core Principles

We strive to follow these fundamental software design principles:

*   **SOLID**:
    *   **S**ingle **R**esponsibility **P**rinciple (SRP): Each module, struct, or function should have one specific responsibility.
    *   **O**pen/**C**losed **P**rinciple (OCP): Software entities (structs, modules, functions) should be open for extension but closed for modification. This often involves using traits (interfaces) and composition.
    *   **L**iskov **S**ubstitution **P**rinciple (LSP): Subtypes must be substitutable for their base types. In Rust, this often relates to how traits are implemented and used.
    *   **I**nterface **S**egregation **P**rinciple (ISP): Clients should not be forced to depend on interfaces (traits) they do not use. Prefer smaller, more specific traits.
    *   **D**ependency **I**nversion **P**rinciple (DIP): High-level modules should not depend on low-level modules. Both should depend on abstractions (traits). Abstractions should not depend on details; details should depend on abstractions.

*   **DRY (Don't Repeat Yourself)**: Avoid duplication of code. Use functions, modules, and generics to reuse logic.

*   **KISS (Keep It Simple, Stupid)**: Strive for simplicity in design and implementation. Avoid unnecessary complexity.

*   **YAGNI (You Ain't Gonna Need It)**: Do not add functionality until it is necessary.

## 2. Sandi Metz's Rules

Inspired by Sandi Metz's rules for object-oriented design, we adapt them for Rust:

1.  **Struct/Enum Size (LoC Limit)**: Strive to keep structs, enums, and their `impl` blocks relatively small. As a guideline, the combined lines of code for a struct/enum definition and its associated `impl` blocks should ideally not exceed **100 lines**. This encourages breaking down complex entities into smaller, more focused ones.
2.  **Function Size (LoC Limit)**: Functions should be very small. Aim for functions to be no longer than **5 lines of code**. This promotes readability, testability, and single responsibility.
3.  **Function Parameters**: Functions should have a small number of parameters. Aim for no more than **3-4 parameters**. If more are needed, consider grouping parameters into a dedicated struct or using a builder pattern.
4.  **(No Direct Rust Equivalent to "Controller Can Only Instantiate One Object")**: While this rule is specific to MVC controllers, the underlying principle is about limiting responsibilities. In a Rust context, this means a function or method that coordinates or constructs objects should have a very focused scope of what it creates or manages.

## 3. Design Patterns

While not strictly enforced for all situations, consider using appropriate design patterns to solve common problems. Some patterns that might be relevant for this project include:

*   **Builder**: For constructing complex objects (e.g., tokenizer configuration) step-by-step.
*   **Strategy**: For defining a family of algorithms (e.g., different tokenization methods, patching strategies) and making them interchangeable.
*   **Iterator**: For processing sequences of data, especially relevant for token streams and input chunks.
*   **Adapter**: For making incompatible interfaces work together.
*   **Decorator**: For adding responsibilities to objects dynamically.

Favor composition over inheritance where applicable (Rust's trait system naturally encourages this).

## 4. Formatting and Linting

*   **Formatting**: All code must be formatted using `cargo fmt`. This will be checked in the CI pipeline.
*   **Linting**: Code should adhere to `clippy` lints. Run `cargo clippy -- -D warnings` to catch and fix issues. The CI pipeline will also enforce this.

## 5. Testing

*   Write unit tests for individual functions and modules.
*   Write integration tests for CLI behavior and library usage.
*   Tests should also adhere to the coding standards (e.g., small, focused test functions).

## 6. Modularity and Project Structure

*   The project will be structured with a core library crate and a thin binary crate.
*   The core library will be divided into modules, each with a clear responsibility (e.g., `io`, `tokenizer`, `chunking`).
*   Modules should be loosely coupled and highly cohesive.

## 7. Asynchronous Programming (Tokio)

*   Follow best practices for asynchronous Rust with Tokio.
*   Be mindful of blocking operations within async contexts. Use `spawn_blocking` where necessary.
*   Manage tasks and resources carefully to avoid leaks or deadlocks.

## 8. Error Handling

*   Use Rust's `Result` type for functions that can fail.
*   Define specific error types where appropriate, or use established error handling crates (e.g., `thiserror`, `anyhow`) if they simplify error management significantly and consistently.
*   Provide meaningful error messages.

## Contribution Workflow

*   Follow standard Git practices: branch, commit, push, pull request.
*   Ensure your code passes all CI checks before merging.
*   Engage in code reviews to maintain quality and share knowledge.

This document is a living guide and may be updated as the project evolves.
