# Contributing to Kiren

Thank you for your interest in contributing to Kiren! This document provides guidelines and information for contributors.

## 🚀 Quick Start

### Prerequisites

- **Rust 1.70+** - [Install from rustup.rs](https://rustup.rs/)
- **Git** - For version control
- **Basic knowledge of JavaScript and Rust**

### Development Setup

```bash
# 1. Fork the repository on GitHub
# 2. Clone your fork
git clone https://github.com/YOUR_USERNAME/kiren.git
cd kiren

# 3. Build in development mode
cargo build

# 4. Run examples to verify setup
cargo run examples/hello.js

# 5. Run tests
cargo test
```

## 🛠️ Development Workflow

### Making Changes

```bash
# 1. Create a feature branch
git checkout -b feature/your-feature-name

# 2. Make your changes
# Edit src/ files as needed

# 3. Test your changes
cargo test
cargo run examples/hello.js

# 4. Build release version
cargo build --release

# 5. Run benchmarks (if performance-related)
cd benchmarks && ./run-benchmarks.sh
```

### Code Style

- Follow standard Rust formatting: `cargo fmt`
- Run clippy for lints: `cargo clippy`
- Add tests for new functionality
- Update documentation for new APIs

## 📁 Project Structure

```
kiren/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── api/              # JavaScript API implementations
│   │   ├── console.rs    # Console API (console.log, etc.)
│   │   ├── timers.rs     # Timer APIs (setTimeout, etc.)
│   │   ├── fetch.rs      # HTTP client (fetch API)
│   │   ├── filesystem.rs # File system operations
│   │   ├── process.rs    # Process API (env, argv, etc.)
│   │   ├── http.rs       # HTTP server API
│   │   └── mod.rs        # API module exports
│   └── runtime/
│       ├── engine.rs     # V8 JavaScript engine integration
│       └── mod.rs        # Runtime module exports
├── examples/             # Example JavaScript applications
├── benchmarks/           # Performance benchmarks
├── docs/                 # API documentation
└── target/               # Build artifacts (auto-generated)
```

## 🎯 Areas for Contribution

### High Priority

1. **Performance Optimization**
   - V8 engine integration improvements
   - Startup time optimization
   - Memory usage reduction

2. **Stability Fixes**
   - Fix segmentation faults
   - Improve error handling
   - Add comprehensive testing

3. **Core Features**
   - Timer callback execution
   - ES Modules support
   - Better HTTP server routing

### Medium Priority

1. **Developer Experience**
   - Better error messages with stack traces
   - REPL improvements
   - Hot reload for development

2. **API Expansion**
   - More HTTP server features
   - Crypto API
   - URL parsing utilities

3. **Documentation**
   - API reference improvements
   - More examples
   - Performance guides

### Low Priority

1. **Advanced Features**
   - TypeScript execution
   - WebAssembly support
   - Package manager integration

## 🧪 Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test runtime

# Run with output
cargo test -- --nocapture
```

### Adding Tests

- Unit tests: Add to the same file as the function being tested
- Integration tests: Add to `tests/` directory
- Example tests: Ensure examples in `examples/` work correctly

### Performance Testing

```bash
cd benchmarks
./run-benchmarks.sh
```

## 📝 Documentation

### API Documentation

- Document all public functions with `///` comments
- Include examples in documentation
- Update `docs/api/` markdown files for new APIs

### Examples

- Add working examples for new features
- Keep examples simple and focused
- Test examples regularly

## 🔄 Submission Process

### Pull Request Guidelines

1. **Clear Description**
   - Explain what your change does
   - Reference any related issues
   - Include before/after behavior

2. **Testing**
   - Ensure all tests pass
   - Add tests for new functionality
   - Verify examples still work

3. **Documentation**
   - Update relevant documentation
   - Add API docs for new features
   - Update CHANGELOG.md

### Commit Message Format

```
type(scope): brief description

Longer explanation if needed.

Fixes #123
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix  
- `docs`: Documentation changes
- `perf`: Performance improvements
- `test`: Adding tests
- `refactor`: Code refactoring

**Examples:**
```
feat(http): add static file serving
fix(timers): resolve segfault in setTimeout
docs(api): update fetch API documentation
perf(runtime): optimize V8 initialization
```

## 🐛 Bug Reports

### Before Reporting

1. Check existing issues
2. Reproduce with minimal example
3. Test with latest version

### Report Template

```markdown
## Bug Description
Brief description of the issue

## Steps to Reproduce
1. Step one
2. Step two
3. See error

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- Kiren version: 
- OS: 
- Rust version: 

## Minimal Example
```javascript
// Minimal code that reproduces the issue
```

## 💡 Feature Requests

### Request Template

```markdown
## Feature Description
Clear description of the requested feature

## Use Case
Why would this feature be useful?

## Proposed API
```javascript
// Example of how the API might look
```

## Alternative Solutions
Any alternative approaches considered
```

## 🤝 Code of Conduct

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Respect different perspectives and experiences

## 📞 Getting Help

- **Discord**: [Join our community](https://discord.gg/kiren) (coming soon)
- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion

## 🎉 Recognition

Contributors will be:
- Listed in CHANGELOG.md for their contributions
- Mentioned in release notes for significant features
- Added to a CONTRIBUTORS.md file

Thank you for contributing to Kiren! 🚀