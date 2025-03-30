# Contributing to msft-recon-rs

Thank you for considering contributing to msft-recon-rs! This document outlines the process for contributing to the project.

## Code of Conduct

This project adheres to the Contributor Covenant Code of Conduct. By participating, you are expected to uphold this code.

## Rust Coding Standards

This project follows Rust Clean Architecture principles and best practices:

1. **Ownership & Borrowing:** Leverage Rust's ownership system effectively. Use explicit lifetimes only when necessary.
2. **Modularity:** Structure code into logical modules with clear interfaces.
3. **Explicit Error Handling:** Use `Result<T, E>` for recoverable errors with meaningful messages.
4. **Immutability by Default:** Prefer immutable variables and data structures.
5. **Traits for Abstraction:** Define behaviors through traits to promote loose coupling.

## Development Workflow

1. **Fork the repository** and create your branch from `master`.
2. **Install development dependencies**:
   ```bash
   # Install clippy for linting
   rustup component add clippy
   ```
3. **Make your changes**:
   - Write tests for new functionality
   - Ensure existing tests pass
   - Follow the project's code style
   - Keep commits focused and clean
4. **Run tests and linting**:
   ```bash
   cargo test
   cargo clippy --all-features -- -D warnings
   ```
5. **Submit a pull request**:
   - Include a clear description of the changes
   - Link any related issues
   - Update documentation as needed

## Pull Request Process

1. Update the README.md or documentation with details of changes if appropriate.
2. The PR should work for all supported platforms and pass CI checks.
3. The PR needs approval from at least one maintainer before being merged.

## Testing

- Write unit tests for all new functionality.
- Ensure tests are independent and isolated.
- Consider edge cases and error conditions in your tests.

## Reporting Bugs

When reporting bugs, please include:

- A clear description of the issue
- Steps to reproduce the behavior
- Expected vs. actual results
- Version information (Rust version, OS, etc.)

## Feature Requests

Feature requests are welcome! Please provide:

- A clear description of the feature
- The rationale for adding it
- Potential implementation approaches if you have ideas

## Community

We value community contributions and strive to be welcoming to all contributors. Don't hesitate to ask questions if something is unclear.

Thank you for contributing to msft-recon-rs!
