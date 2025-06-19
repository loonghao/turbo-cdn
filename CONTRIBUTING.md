# Contributing to Turbo CDN

Thank you for your interest in contributing to Turbo CDN! We welcome contributions from the community and are grateful for your help in making this project better.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contributing Guidelines](#contributing-guidelines)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)
- [Release Process](#release-process)

## 📜 Code of Conduct

This project and everyone participating in it is governed by our Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to [hal.long@outlook.com](mailto:hal.long@outlook.com).

### Our Standards

- **Be respectful**: Treat everyone with respect and kindness
- **Be inclusive**: Welcome newcomers and help them get started
- **Be collaborative**: Work together towards common goals
- **Be constructive**: Provide helpful feedback and suggestions
- **Be professional**: Maintain a professional tone in all interactions

## 🚀 Getting Started

### Prerequisites

- Rust 1.70.0 or later
- Git
- A GitHub account

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/turbo-cdn.git
   cd turbo-cdn
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/loonghao/turbo-cdn.git
   ```

## 🛠️ Development Setup

### Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build

# Run tests
cargo test

# Install development tools
cargo install cargo-watch
cargo install cargo-audit
cargo install cargo-tarpaulin
```

### Development Workflow

```bash
# Start development with auto-reload
cargo watch -x check -x test -x run

# Run with debug logging
RUST_LOG=turbo_cdn=debug cargo run

# Run specific tests
cargo test test_name

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Security audit
cargo audit
```

## 📝 Contributing Guidelines

### Types of Contributions

We welcome several types of contributions:

- **Bug fixes**: Fix issues and improve stability
- **Features**: Add new functionality
- **Documentation**: Improve docs, examples, and guides
- **Performance**: Optimize code and algorithms
- **Tests**: Add or improve test coverage
- **Refactoring**: Improve code quality and maintainability

### Before You Start

1. **Check existing issues**: Look for existing issues or discussions
2. **Create an issue**: For significant changes, create an issue first
3. **Discuss the approach**: Get feedback on your proposed solution
4. **Keep it focused**: One feature/fix per pull request

## 🔄 Pull Request Process

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 2. Make Your Changes

- Write clear, concise commit messages
- Follow the coding standards
- Add tests for new functionality
- Update documentation as needed

### 3. Test Your Changes

```bash
# Run all tests
cargo test

# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check documentation
cargo doc --no-deps
```

### 4. Commit Your Changes

```bash
git add .
git commit -m "feat: add new download source support

- Add support for custom CDN sources
- Implement source validation
- Add comprehensive tests
- Update documentation

Signed-off-by: Your Name <your.email@example.com>"
```

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a pull request on GitHub with:
- Clear title and description
- Reference to related issues
- Screenshots/examples if applicable
- Checklist of completed items

### 6. Code Review Process

- Maintainers will review your PR
- Address feedback promptly
- Keep the PR updated with main branch
- Be patient and responsive

## 🐛 Issue Reporting

### Bug Reports

When reporting bugs, please include:

- **Environment**: OS, Rust version, Turbo CDN version
- **Steps to reproduce**: Clear, step-by-step instructions
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Logs**: Relevant error messages or logs
- **Minimal example**: Code that reproduces the issue

### Feature Requests

For feature requests, please include:

- **Use case**: Why is this feature needed?
- **Proposed solution**: How should it work?
- **Alternatives**: Other solutions you've considered
- **Additional context**: Any other relevant information

## 🎯 Coding Standards

### Rust Style Guide

- Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- Use `cargo fmt` for consistent formatting
- Use `cargo clippy` for linting
- Write idiomatic Rust code

### Code Organization

- Keep modules focused and cohesive
- Use clear, descriptive names
- Add comprehensive documentation
- Include examples in doc comments

### Error Handling

- Use `Result<T, E>` for fallible operations
- Create specific error types
- Provide helpful error messages
- Use `?` operator for error propagation

### Documentation

- Document all public APIs
- Include examples in doc comments
- Keep documentation up to date
- Use clear, concise language

## 🧪 Testing

### Test Categories

1. **Unit Tests**: Test individual functions and modules
2. **Integration Tests**: Test component interactions
3. **Compliance Tests**: Verify legal and compliance requirements
4. **Performance Tests**: Benchmark critical paths

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download_success() {
        // Arrange
        let downloader = create_test_downloader().await;
        
        // Act
        let result = downloader.download("test/repo", "v1.0.0", "file.zip").await;
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### Test Guidelines

- Write tests for all new functionality
- Test both success and failure cases
- Use descriptive test names
- Keep tests focused and independent
- Mock external dependencies

## 📚 Documentation

### Types of Documentation

- **API Documentation**: Rustdoc comments
- **User Guide**: README and examples
- **Developer Guide**: This contributing guide
- **Architecture**: Design decisions and patterns

### Documentation Standards

- Write clear, concise documentation
- Include practical examples
- Keep documentation up to date
- Use proper Markdown formatting

## 🚢 Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

- [ ] All tests pass
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Security audit clean
- [ ] Performance benchmarks stable

## 🤝 Community

### Getting Help

- **GitHub Discussions**: For questions and general discussion
- **GitHub Issues**: For bug reports and feature requests
- **Email**: [hal.long@outlook.com](mailto:hal.long@outlook.com) for security issues

### Recognition

Contributors will be recognized in:
- CHANGELOG.md for significant contributions
- README.md acknowledgments section
- GitHub contributors page

## 📄 License

By contributing to Turbo CDN, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Turbo CDN! Your efforts help make open-source software more accessible to everyone. 🚀
