# takumi-sharp

C# bindings for [Takumi](https://github.com/kane50613/takumi), a high-performance image rendering library.

## Project Structure

- `takumi-native/` - Rust FFI wrapper that compiles to native libraries
- `takumi-sharp/TakumiSharp/` - Main C# library with bindings
- `takumi-sharp/TakumiSharp.Native/` - NuGet package containing native libraries for all platforms

## Building Native Libraries

The project includes build scripts to compile native libraries for multiple platforms.

### Supported Platforms

| .NET RID           | Platform                    | Rust Target                    |
|--------------------|-----------------------------|--------------------------------|
| `win-x64`          | Windows x64                 | `x86_64-pc-windows-msvc`       |
| `win-arm64`        | Windows ARM64               | `aarch64-pc-windows-msvc`      |
| `linux-x64`        | Linux x64 (glibc)           | `x86_64-unknown-linux-gnu`     |
| `linux-arm64`      | Linux ARM64 (glibc)         | `aarch64-unknown-linux-gnu`    |
| `linux-musl-x64`   | Linux x64 (musl/Alpine)     | `x86_64-unknown-linux-musl`    |
| `linux-musl-arm64` | Linux ARM64 (musl/Alpine)   | `aarch64-unknown-linux-musl`   |
| `osx-x64`          | macOS x64 (Intel)           | `x86_64-apple-darwin`          |
| `osx-arm64`        | macOS ARM64 (Apple Silicon) | `aarch64-apple-darwin`         |

### Prerequisites

- [Rust](https://rustup.rs/) with `rustup` for managing targets
- [.NET 10 SDK](https://dotnet.microsoft.com/download) (or later)

### Build Commands

#### macOS/Linux (Bash)

```bash
# Build for current platform (auto-detected)
./scripts/build-natives.sh

# Build for all platforms
./scripts/build-natives.sh --all

# Build for specific platforms
./scripts/build-natives.sh osx-arm64 linux-x64

# Build in debug mode
./scripts/build-natives.sh --debug osx-arm64

# Clean and rebuild
./scripts/build-natives.sh --clean --all
```

#### Windows (PowerShell)

```powershell
# Build for current platform (auto-detected)
.\scripts\build-natives.ps1

# Build for all platforms
.\scripts\build-natives.ps1 -All

# Build for specific platforms
.\scripts\build-natives.ps1 win-x64 linux-x64

# Build in debug mode
.\scripts\build-natives.ps1 -Debug win-x64

# Clean and rebuild
.\scripts\build-natives.ps1 -Clean -All
```

### CI/CD

The GitHub Actions workflow (`.github/workflows/build-natives.yml`) automatically:
1. Builds native libraries for all platforms in parallel
2. Collects all artifacts
3. Optionally packs the NuGet package on main branch pushes

## Development

```bash
# Install Rust targets for cross-compilation
rustup target add x86_64-pc-windows-msvc
rustup target add aarch64-pc-windows-msvc
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
rustup target add aarch64-unknown-linux-musl
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build and run example
./build-and-run-example.sh
```
