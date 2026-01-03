# Building Kiren

This document describes how to build Kiren from source.

## Requirements

### Zig

Kiren requires Zig 0.14.0 or later.

**macOS (Homebrew):**
```bash
brew install zig
```

**Linux:**
```bash
# Download from ziglang.org
wget https://ziglang.org/download/0.14.0/zig-linux-x86_64-0.14.0.tar.xz
tar -xf zig-linux-x86_64-0.14.0.tar.xz
export PATH=$PATH:$PWD/zig-linux-x86_64-0.14.0
```

**Windows:**
```powershell
# Download from ziglang.org or use scoop
scoop install zig
```

Verify installation:
```bash
zig version
# Output: 0.14.0
```

### C Compiler

Zig includes a C compiler, but system headers may be required on some platforms.

**macOS:**
```bash
xcode-select --install
```

**Linux (Debian/Ubuntu):**
```bash
sudo apt install build-essential
```

**Linux (Fedora):**
```bash
sudo dnf install gcc glibc-devel
```

## Building

### Clone the Repository

```bash
git clone https://github.com/user/kiren.git
cd kiren
```

The repository includes QuickJS as a submodule in `deps/quickjs/`.

### Debug Build

For development with debug symbols and runtime checks:

```bash
zig build
```

Output: `zig-out/bin/kiren`

### Release Build

For production use with optimizations:

```bash
zig build -Doptimize=ReleaseFast
```

This produces a smaller, faster binary.

### Small Binary Build

For minimal binary size:

```bash
zig build -Doptimize=ReleaseSmall
```

### Build Options

| Option | Description |
|--------|-------------|
| `-Doptimize=Debug` | Debug build (default) |
| `-Doptimize=ReleaseFast` | Optimized for speed |
| `-Doptimize=ReleaseSmall` | Optimized for size |
| `-Doptimize=ReleaseSafe` | Release with safety checks |

## Running

### Direct Execution

```bash
zig build run -- script.js
```

### Using Built Binary

```bash
./zig-out/bin/kiren script.js
```

### With Arguments

```bash
./zig-out/bin/kiren server.js --port 3000
```

Arguments after the script name are available via `process.argv`.

## Testing

### Zig Unit Tests

```bash
zig build test
```

### JavaScript Tests

```bash
# Run individual test
./zig-out/bin/kiren tests/api/buffer.test.js

# Run all API tests
for f in tests/api/*.test.js; do
  echo "Running $f"
  ./zig-out/bin/kiren "$f"
done

# Run HTTP tests (starts servers)
./zig-out/bin/kiren tests/http/simple.test.js
```

## Cross-Compilation

Zig supports cross-compilation out of the box.

### Linux (from macOS)

```bash
zig build -Dtarget=x86_64-linux-gnu -Doptimize=ReleaseFast
```

### Windows (from macOS/Linux)

```bash
zig build -Dtarget=x86_64-windows-gnu -Doptimize=ReleaseFast
```

### ARM64 Linux

```bash
zig build -Dtarget=aarch64-linux-gnu -Doptimize=ReleaseFast
```

### Available Targets

List all available targets:
```bash
zig targets | jq '.native'
```

Common targets:
- `x86_64-linux-gnu`
- `aarch64-linux-gnu`
- `x86_64-macos`
- `aarch64-macos`
- `x86_64-windows-gnu`

## Installation

### Local Installation

```bash
# Build release version
zig build -Doptimize=ReleaseFast

# Copy to local bin
cp zig-out/bin/kiren ~/.local/bin/

# Or system-wide (requires sudo)
sudo cp zig-out/bin/kiren /usr/local/bin/
```

### Verify Installation

```bash
kiren --version
```

## Troubleshooting

### "error: libc headers not found"

Install system development headers:

```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt install libc6-dev

# Fedora
sudo dnf install glibc-devel
```

### QuickJS Compilation Errors

QuickJS requires specific C compiler flags. These are configured in `build.zig`:

```zig
const c_flags = [_][]const u8{
    "-DCONFIG_VERSION=\"2024-01-13\"",
    "-DCONFIG_BIGNUM",
    "-D_GNU_SOURCE",
    "-fno-sanitize=undefined",
    "-fwrapv",
};
```

### Build Cache Issues

Clear the build cache:

```bash
rm -rf .zig-cache zig-out
zig build
```

### Memory Issues During Build

Large projects may require more memory. If build fails with memory errors:

```bash
# Reduce parallel jobs
zig build -j1
```

## Development Workflow

### Incremental Builds

Zig automatically performs incremental builds. After initial compilation, subsequent builds only recompile changed files.

### Watch Mode

For continuous development, use a file watcher:

```bash
# Using entr (install via package manager)
find src -name "*.zig" | entr -c zig build run -- examples/server.js

# Using watchexec
watchexec -e zig "zig build run -- examples/server.js"
```

### Debugging

Build with debug symbols:

```bash
zig build
```

Use lldb or gdb:

```bash
lldb ./zig-out/bin/kiren -- script.js
```

## Binary Size

Typical binary sizes:

| Build Type | Size |
|------------|------|
| Debug | ~8 MB |
| ReleaseFast | ~2 MB |
| ReleaseSmall | ~1.5 MB |

To further reduce size:

```bash
# Build small
zig build -Doptimize=ReleaseSmall

# Strip debug symbols (if any remain)
strip zig-out/bin/kiren
```

## Continuous Integration

Example GitHub Actions workflow:

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Zig
        uses: goto-bus-stop/setup-zig@v2
        with:
          version: 0.14.0

      - name: Build
        run: zig build -Doptimize=ReleaseFast

      - name: Test
        run: zig build test
```
