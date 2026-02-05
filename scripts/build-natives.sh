#!/bin/bash
set -e

# TakumiSharp Native Build Script
# This script compiles the Rust native library for multiple platforms
# and copies them to the TakumiSharp.Native runtimes directory.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
NATIVE_PROJECT_DIR="$ROOT_DIR/takumi-native"
OUTPUT_DIR="$ROOT_DIR/takumi-sharp/TakumiSharp.Native/runtimes"

log_info() {
    echo "[INFO] $1"
}

log_warn() {
    echo "[WARN] $1"
}

log_error() {
    echo "[ERROR] $1"
}

# Function to get Rust target for a given RID
get_rust_target() {
    case "$1" in
        win-x64)          echo "x86_64-pc-windows-msvc" ;;
        win-arm64)        echo "aarch64-pc-windows-msvc" ;;
        linux-x64)        echo "x86_64-unknown-linux-gnu" ;;
        linux-arm64)      echo "aarch64-unknown-linux-gnu" ;;
        linux-musl-x64)   echo "x86_64-unknown-linux-musl" ;;
        linux-musl-arm64) echo "aarch64-unknown-linux-musl" ;;
        osx-x64)          echo "x86_64-apple-darwin" ;;
        osx-arm64)        echo "aarch64-apple-darwin" ;;
        *)                echo "" ;;
    esac
}

# Function to get library name for a given RID
get_lib_name() {
    case "$1" in
        win-x64|win-arm64)                                       echo "takumi.dll" ;;
        linux-x64|linux-arm64|linux-musl-x64|linux-musl-arm64)   echo "libtakumi.so" ;;
        osx-x64|osx-arm64)                                       echo "libtakumi.dylib" ;;
        *)                                                       echo "" ;;
    esac
}

# Function to check if a target is valid
is_valid_target() {
    case "$1" in
        win-x64|win-arm64|linux-x64|linux-arm64|linux-musl-x64|linux-musl-arm64|osx-x64|osx-arm64) return 0 ;;
        *) return 1 ;;
    esac
}

ALL_TARGETS="win-x64 win-arm64 linux-x64 linux-arm64 linux-musl-x64 linux-musl-arm64 osx-x64 osx-arm64"

# Parse arguments
BUILD_TARGETS=""
RELEASE_MODE=true
CLEAN_BUILD=false

print_usage() {
    echo "Usage: $0 [OPTIONS] [TARGETS...]"
    echo ""
    echo "Build native libraries for TakumiSharp."
    echo ""
    echo "Options:"
    echo "  --debug         Build in debug mode (default: release)"
    echo "  --clean         Clean build artifacts before building"
    echo "  --all           Build for all supported targets"
    echo "  --help, -h      Show this help message"
    echo ""
    echo "Supported targets:"
    echo "  win-x64           Windows x64"
    echo "  win-arm64         Windows ARM64"
    echo "  linux-x64         Linux x64 (glibc)"
    echo "  linux-arm64       Linux ARM64 (glibc)"
    echo "  linux-musl-x64    Linux x64 (musl/Alpine)"
    echo "  linux-musl-arm64  Linux ARM64 (musl/Alpine)"
    echo "  osx-x64           macOS x64 (Intel)"
    echo "  osx-arm64         macOS ARM64 (Apple Silicon)"
    echo ""
    echo "Examples:"
    echo "  $0 --all                    # Build all targets in release mode"
    echo "  $0 osx-arm64                # Build only for macOS ARM64"
    echo "  $0 --debug linux-x64        # Build Linux x64 in debug mode"
    echo "  $0 win-x64 linux-x64        # Build for Windows and Linux x64"
}

while [ $# -gt 0 ]; do
    case $1 in
        --debug)
            RELEASE_MODE=false
            shift
            ;;
        --clean)
            CLEAN_BUILD=true
            shift
            ;;
        --all)
            BUILD_TARGETS="$ALL_TARGETS"
            shift
            ;;
        --help|-h)
            print_usage
            exit 0
            ;;
        *)
            if is_valid_target "$1"; then
                BUILD_TARGETS="$BUILD_TARGETS $1"
            else
                log_error "Unknown target or option: $1"
                print_usage
                exit 1
            fi
            shift
            ;;
    esac
done

# Trim leading whitespace
BUILD_TARGETS="$(echo "$BUILD_TARGETS" | sed 's/^ *//')"

# Default to current platform if no targets specified
if [ -z "$BUILD_TARGETS" ]; then
    case "$(uname -s)-$(uname -m)" in
        Darwin-arm64)
            BUILD_TARGETS="osx-arm64"
            ;;
        Darwin-x86_64)
            BUILD_TARGETS="osx-x64"
            ;;
        Linux-x86_64)
            BUILD_TARGETS="linux-x64"
            ;;
        Linux-aarch64)
            BUILD_TARGETS="linux-arm64"
            ;;
        *)
            log_error "Could not detect current platform. Please specify a target."
            print_usage
            exit 1
            ;;
    esac
    log_info "No target specified, defaulting to: $BUILD_TARGETS"
fi

# Ensure we're in the native project directory
cd "$NATIVE_PROJECT_DIR"

# Clean if requested
if [ "$CLEAN_BUILD" = true ]; then
    log_info "Cleaning build artifacts..."
    cargo clean
fi

# Build profile
if [ "$RELEASE_MODE" = true ]; then
    BUILD_PROFILE="release"
    CARGO_FLAGS="--release"
else
    BUILD_PROFILE="debug"
    CARGO_FLAGS=""
fi

log_info "Build profile: $BUILD_PROFILE"
log_info "Targets: $BUILD_TARGETS"
echo ""

# Track build results
SUCCESS_TARGETS=""
FAILED_TARGETS=""

# Build for each target
for rid in $BUILD_TARGETS; do
    rust_target=$(get_rust_target "$rid")
    lib_name=$(get_lib_name "$rid")
    
    log_info "Building for $rid (Rust target: $rust_target)..."
    
    # Check if the target is installed
    if ! rustup target list --installed | grep -q "^$rust_target$"; then
        log_warn "Target $rust_target is not installed. Installing..."
        if ! rustup target add "$rust_target"; then
            log_error "Failed to install target $rust_target"
            FAILED_TARGETS="$FAILED_TARGETS $rid"
            continue
        fi
    fi
    
    # Build
    if cargo build $CARGO_FLAGS --target "$rust_target"; then
        # Determine source path
        SOURCE_PATH="$ROOT_DIR/target/$rust_target/$BUILD_PROFILE/$lib_name"
        
        # Create output directory
        DEST_DIR="$OUTPUT_DIR/$rid/native"
        mkdir -p "$DEST_DIR"
        
        # Copy the library
        if [ -f "$SOURCE_PATH" ]; then
            cp "$SOURCE_PATH" "$DEST_DIR/"
            log_info "Copied $lib_name to $DEST_DIR/"
            SUCCESS_TARGETS="$SUCCESS_TARGETS $rid"
        else
            log_error "Built library not found at: $SOURCE_PATH"
            FAILED_TARGETS="$FAILED_TARGETS $rid"
        fi
    else
        log_error "Build failed for $rid"
        FAILED_TARGETS="$FAILED_TARGETS $rid"
    fi
    
    echo ""
done

# Print summary
echo ""
echo "========================================"
echo "Build Summary"
echo "========================================"
FAILED_COUNT=0
for rid in $BUILD_TARGETS; do
    if echo "$SUCCESS_TARGETS" | grep -q "$rid"; then
        echo "  $rid: ${GREEN}SUCCESS${NC}"
    else
        echo "  $rid: ${RED}FAILED${NC}"
        FAILED_COUNT=$((FAILED_COUNT + 1))
    fi
done
echo "========================================"

if [ $FAILED_COUNT -gt 0 ]; then
    log_error "$FAILED_COUNT target(s) failed to build"
    exit 1
else
    log_info "All targets built successfully!"
fi
