# TakumiSharp Native Build Script for Windows
# This script compiles the Rust native library for multiple platforms
# and copies them to the TakumiSharp.Native runtimes directory.

param(
    [switch]$Debug,
    [switch]$Clean,
    [switch]$All,
    [switch]$Help,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Targets
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
$NativeProjectDir = Join-Path $RootDir "takumi-native"
$OutputDir = Join-Path $RootDir "takumi-sharp" "TakumiSharp.Native" "runtimes"

# Map of .NET RID to Rust target triple and library file name
$TargetMap = @{
    "win-x64"          = @{ RustTarget = "x86_64-pc-windows-msvc"; LibName = "takumi.dll" }
    "win-arm64"        = @{ RustTarget = "aarch64-pc-windows-msvc"; LibName = "takumi.dll" }
    "linux-x64"        = @{ RustTarget = "x86_64-unknown-linux-gnu"; LibName = "libtakumi.so" }
    "linux-arm64"      = @{ RustTarget = "aarch64-unknown-linux-gnu"; LibName = "libtakumi.so" }
    "linux-musl-x64"   = @{ RustTarget = "x86_64-unknown-linux-musl"; LibName = "libtakumi.so" }
    "linux-musl-arm64" = @{ RustTarget = "aarch64-unknown-linux-musl"; LibName = "libtakumi.so" }
    "osx-x64"          = @{ RustTarget = "x86_64-apple-darwin"; LibName = "libtakumi.dylib" }
    "osx-arm64"        = @{ RustTarget = "aarch64-apple-darwin"; LibName = "libtakumi.dylib" }
}

function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Show-Usage {
    Write-Host @"
Usage: build-natives.ps1 [OPTIONS] [TARGETS...]

Build native libraries for TakumiSharp.

Options:
  -Debug          Build in debug mode (default: release)
  -Clean          Clean build artifacts before building
  -All            Build for all supported targets
  -Help           Show this help message

Supported targets:
  win-x64           Windows x64
  win-arm64         Windows ARM64
  linux-x64         Linux x64 (glibc)
  linux-arm64       Linux ARM64 (glibc)
  linux-musl-x64    Linux x64 (musl/Alpine)
  linux-musl-arm64  Linux ARM64 (musl/Alpine)
  osx-x64           macOS x64 (Intel)
  osx-arm64         macOS ARM64 (Apple Silicon)

Examples:
  .\build-natives.ps1 -All                    # Build all targets in release mode
  .\build-natives.ps1 win-x64                 # Build only for Windows x64
  .\build-natives.ps1 -Debug win-x64          # Build Windows x64 in debug mode
  .\build-natives.ps1 win-x64 linux-x64       # Build for Windows and Linux x64
"@
}

if ($Help) {
    Show-Usage
    exit 0
}

# Determine build targets
$BuildTargets = @()

if ($All) {
    $BuildTargets = @("win-x64", "win-arm64", "linux-x64", "linux-arm64", "linux-musl-x64", "linux-musl-arm64", "osx-x64", "osx-arm64")
}
elseif ($Targets.Count -gt 0) {
    foreach ($target in $Targets) {
        if ($TargetMap.ContainsKey($target)) {
            $BuildTargets += $target
        }
        else {
            Write-Error "Unknown target: $target"
            Show-Usage
            exit 1
        }
    }
}
else {
    # Default to current platform
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    if ($IsWindows -or $env:OS -eq "Windows_NT") {
        if ($arch -eq "Arm64") {
            $BuildTargets = @("win-arm64")
        }
        else {
            $BuildTargets = @("win-x64")
        }
    }
    elseif ($IsMacOS) {
        if ($arch -eq "Arm64") {
            $BuildTargets = @("osx-arm64")
        }
        else {
            $BuildTargets = @("osx-x64")
        }
    }
    elseif ($IsLinux) {
        if ($arch -eq "Arm64") {
            $BuildTargets = @("linux-arm64")
        }
        else {
            $BuildTargets = @("linux-x64")
        }
    }
    else {
        Write-Error "Could not detect current platform. Please specify a target."
        Show-Usage
        exit 1
    }
    Write-Info "No target specified, defaulting to: $($BuildTargets -join ', ')"
}

# Navigate to native project directory
Push-Location $NativeProjectDir

try {
    # Clean if requested
    if ($Clean) {
        Write-Info "Cleaning build artifacts..."
        cargo clean
    }

    # Build profile
    $BuildProfile = if ($Debug) { "debug" } else { "release" }
    $CargoFlags = if ($Debug) { @() } else { @("--release") }

    Write-Info "Build profile: $BuildProfile"
    Write-Info "Targets: $($BuildTargets -join ', ')"
    Write-Host ""

    $BuildResults = @{}
    $FailedCount = 0

    foreach ($rid in $BuildTargets) {
        $targetInfo = $TargetMap[$rid]
        $rustTarget = $targetInfo.RustTarget
        $libName = $targetInfo.LibName

        Write-Info "Building for $rid (Rust target: $rustTarget)..."

        # Check if target is installed
        $installedTargets = rustup target list --installed
        if ($installedTargets -notcontains $rustTarget) {
            Write-Warn "Target $rustTarget is not installed. Installing..."
            rustup target add $rustTarget
            if ($LASTEXITCODE -ne 0) {
                Write-Error "Failed to install target $rustTarget"
                $BuildResults[$rid] = "FAILED (target not available)"
                $FailedCount++
                continue
            }
        }

        # Build
        $buildArgs = @("build") + $CargoFlags + @("--target", $rustTarget)
        cargo @buildArgs
        
        if ($LASTEXITCODE -eq 0) {
            # Determine source path
            $sourcePath = Join-Path $RootDir "target" $rustTarget $BuildProfile $libName

            # Create output directory
            $destDir = Join-Path $OutputDir $rid "native"
            New-Item -ItemType Directory -Force -Path $destDir | Out-Null

            # Copy the library
            if (Test-Path $sourcePath) {
                Copy-Item $sourcePath -Destination $destDir -Force
                Write-Info "Copied $libName to $destDir"
                $BuildResults[$rid] = "SUCCESS"
            }
            else {
                Write-Error "Built library not found at: $sourcePath"
                $BuildResults[$rid] = "FAILED (library not found)"
                $FailedCount++
            }
        }
        else {
            Write-Error "Build failed for $rid"
            $BuildResults[$rid] = "FAILED (build error)"
            $FailedCount++
        }

        Write-Host ""
    }

    # Print summary
    Write-Host ""
    Write-Host "========================================"
    Write-Host "Build Summary"
    Write-Host "========================================"
    foreach ($rid in $BuildTargets) {
        $result = $BuildResults[$rid]
        if ($result -eq "SUCCESS") {
            Write-Host "  ${rid}: " -NoNewline
            Write-Host $result -ForegroundColor Green
        }
        else {
            Write-Host "  ${rid}: " -NoNewline
            Write-Host $result -ForegroundColor Red
        }
    }
    Write-Host "========================================"

    if ($FailedCount -gt 0) {
        Write-Error "$FailedCount target(s) failed to build"
        exit 1
    }
    else {
        Write-Info "All targets built successfully!"
    }
}
finally {
    Pop-Location
}
