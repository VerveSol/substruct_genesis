# Contributing to substruct-genesis

Thank you for your interest in contributing to `substruct-genesis`! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contributing Guidelines](#contributing-guidelines)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Documentation](#documentation)
- [Release Process](#release-process)

## Code of Conduct

This project adheres to the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). By participating, you are expected to uphold this code.

## Getting Started

### Prerequisites

- **Rust 1.85.0+** (Rust Edition 2024)
- **Git** for version control
- **Basic understanding** of Rust procedural macros
- **Familiarity** with `syn`, `quote`, and `proc-macro2` crates

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/substruct_genesis.git
   cd substruct_genesis
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/VerveSol/substruct_genesis.git
   ```

## Development Setup

### 1. Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify Rust version (must be 1.85.0+)
rustc --version
```

### 2. Build the Project

```bash
# Build the project
cargo build

# Run tests
cargo test

# Check for linting issues
cargo clippy

# Format code
cargo fmt
```

### 3. Verify Everything Works

```bash
# Run the full test suite
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test categories
cargo test --test basic_functionality
cargo test --test field_types
cargo test --test complex_scenarios
```

## Contributing Guidelines

### Types of Contributions

We welcome several types of contributions:

#### **Bug Fixes**
- Fix issues reported in GitHub issues
- Ensure all existing tests pass
- Add tests for the bug fix if applicable

#### **New Features**
- Implement new field types or attributes
- Add new utility methods to generated substructs
- Enhance error messages and diagnostics

#### **Documentation**
- Improve README.md
- Add code examples
- Enhance inline documentation
- Write tutorials or guides

#### **Testing**
- Add test cases for edge cases
- Improve test coverage
- Add integration tests
- Performance benchmarks

#### **Infrastructure**
- CI/CD improvements
- Build system enhancements
- Dependency updates

### Code Style

#### **Rust Formatting**
- Use `cargo fmt` to format code
- Follow standard Rust conventions
- Use meaningful variable and function names

#### **Documentation**
- Document all public APIs with `///` comments
- Include examples in documentation
- Use `rust,ignore` for doctests that can't compile standalone

#### **Error Handling**
- Use `proc_macro_error` for user-friendly error messages
- Provide clear error context
- Include suggestions for fixes when possible

#### **Testing**
- Write tests for all new functionality
- Use descriptive test names
- Group related tests logically
- Test both success and failure cases

## Pull Request Process

### 1. **Create a Feature Branch**

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number
```

### 2. **Make Your Changes**

- Write your code following the style guidelines
- Add tests for new functionality
- Update documentation as needed
- Ensure all tests pass

### 3. **Commit Your Changes**

```bash
# Stage your changes
git add .

# Commit with a descriptive message
git commit -m "Add support for custom field types

- Implement new field type attribute
- Add comprehensive tests
- Update documentation with examples
- Fix issue #123"
```

**Commit Message Format:**
- Use imperative mood ("Add feature" not "Added feature")
- First line should be 50 characters or less
- Include issue number if applicable
- Provide detailed description in body if needed

### 4. **Push and Create PR**

```bash
# Push your branch
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub with:
- **Clear title** describing the change
- **Detailed description** of what was changed and why
- **Reference to related issues** using "Fixes #123" or "Closes #123"
- **Screenshots** if UI changes are involved

### 5. **PR Review Process**

- Maintainers will review your PR
- Address any feedback promptly
- Make additional commits if requested
- Keep the PR focused on a single change

## Issue Reporting

### Before Creating an Issue

1. **Search existing issues** to avoid duplicates
2. **Check if it's already fixed** in the latest version
3. **Verify it's a bug** and not expected behavior

### Creating a Good Issue

#### **Bug Reports**
```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Expected behavior**
What you expected to happen.

**Code example**
```rust
// Minimal code that reproduces the issue
```

**Environment:**
- OS: [e.g. macOS, Linux, Windows]
- Rust version: [e.g. 1.85.0]
- substruct-genesis version: [e.g. 0.1.0]

**Additional context**
Any other context about the problem.
```

#### **Feature Requests**
```markdown
**Is your feature request related to a problem?**
A clear description of what the problem is.

**Describe the solution you'd like**
A clear description of what you want to happen.

**Describe alternatives you've considered**
Alternative solutions or workarounds.

**Additional context**
Any other context or examples about the feature request.
```

## Development Workflow

### **Project Structure**

```
substruct_genesis/
├── src/
│   ├── lib.rs              # Main macro implementation
│   ├── generator.rs         # Code generation logic
│   └── processor/
│       ├── mod.rs          # Module declarations
│       ├── attributes.rs   # Attribute parsing
│       └── fields.rs       # Field processing
├── tests/                  # Integration tests
├── Cargo.toml             # Project configuration
└── README.md              # Project documentation
```

### **Key Components**

#### **lib.rs**
- Main procedural macro entry point
- Orchestrates the code generation process
- Handles error reporting

#### **generator.rs**
- Generates the final Rust code
- Implements all utility methods
- Handles trait implementations

#### **processor/**
- **attributes.rs**: Parses struct and field attributes
- **fields.rs**: Processes individual fields and determines types

### **Adding New Features**

#### **New Field Types**
1. Add field type to `FieldKind` enum in `processor/fields.rs`
2. Implement handling in `handle_field` function
3. Add code generation in `generator.rs`
4. Add comprehensive tests
5. Update documentation

#### **New Utility Methods**
1. Add method implementation in `generator.rs`
2. Add comprehensive tests
3. Update documentation with examples
4. Ensure backward compatibility

## Testing

### **Test Structure**

The project has 8 test files covering different aspects:

- **basic_functionality.rs** - Core macro functionality
- **field_types.rs** - Different field type handling
- **configuration.rs** - Attribute parsing and configuration
- **complex_scenarios.rs** - Complex nested structures
- **integration.rs** - Multiple features working together
- **error_handling.rs** - Error cases and validation
- **real_world.rs** - Real-world usage patterns
- **edge_cases.rs** - Boundary conditions

### **Writing Tests**

#### **Test Naming Convention**
```rust
#[test]
fn test_feature_description() {
    // Test implementation
}
```

#### **Test Structure**
```rust
#[test]
fn test_new_feature() {
    // 1. Setup
    let input = /* test data */;
    
    // 2. Execute
    let result = /* call function */;
    
    // 3. Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_value);
}
```

#### **Running Tests**
```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test basic_functionality

# Run specific test
cargo test test_new_feature

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release
```

## Documentation

### **Documentation Standards**

#### **Inline Documentation**
```rust
/// Creates a new substruct with the specified field values.
///
/// # Arguments
///
/// * `field1` - Description of field1
/// * `field2` - Description of field2
///
/// # Examples
///
/// ```rust,ignore
/// let update = MySubstruct::new(
///     Some("value1".to_string()),
///     Some(true),
/// );
/// ```
pub fn new(field1: Option<String>, field2: Option<bool>) -> Self {
    // Implementation
}
```

#### **README Updates**
- Update examples when adding new features
- Add new sections for new functionality
- Keep the quick start example current
- Update test counts and coverage information

## Getting Help

### **Resources**

- **GitHub Issues**: For bug reports and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Rust Documentation**: [doc.rust-lang.org](https://doc.rust-lang.org/)
- **Procedural Macros Guide**: [Rust Book Chapter 19.6](https://doc.rust-lang.org/book/ch19-06-macros.html)

### **Community**

- Be respectful and constructive
- Help others when you can
- Share knowledge and best practices
- Follow the Rust Code of Conduct
