# Changelog

All notable changes to Kiren will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-06-17

### Added
- **Core Runtime**
  - V8 JavaScript engine integration
  - Command-line interface for executing JavaScript files
  - Interactive REPL mode with `.exit` and `.help` commands

- **Built-in APIs**
  - Console API with `console.log()` support
  - Timer APIs: `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`
  - Fetch API for HTTP requests (Promise-based)
  - File System API: `fs.readFile`, `fs.writeFile`, `fs.exists`, `fs.mkdir`
  - Process API: `process.env`, `process.argv`, `process.cwd()`, `process.exit()`

- **Performance Features**
  - Rust-based runtime for memory safety and performance
  - Tokio async runtime for non-blocking I/O
  - UUID-based timer management for thread safety
  - Optimized V8 initialization

- **Documentation**
  - Comprehensive API documentation in English
  - Getting Started guide
  - Performance optimization guide
  - REPL usage documentation
  - Code examples for all APIs

### Technical Details
- Built with Rust 1.70+
- V8 JavaScript engine 0.84
- Tokio async runtime
- Support for modern JavaScript features (ES2022+)
- Cross-platform compatibility (macOS, Linux, Windows)

### Known Limitations
- Timer callbacks don't execute (IDs are generated correctly)
- ES Modules not yet supported
- No TypeScript support yet
- Limited error messages and stack traces

## [Unreleased]

### Planned
- Enhanced error handling with detailed stack traces
- ES Modules (import/export) support
- TypeScript execution support
- HTTP Server API
- Worker Threads support
- Package manager integration