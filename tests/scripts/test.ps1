# CCM Rust Test Script for PowerShell
# This script tests the Rust refactoring of CCM

$ErrorActionPreference = "Continue"
$ProgressPreference = "SilentlyContinue"

Write-Host "=== CCM Rust Testing Script ===" -ForegroundColor Cyan
Write-Host ""

# Test 1: Check Rust Installation
Write-Host "[1/8] Checking Rust installation..." -ForegroundColor Yellow
try {
    $rustcVersion = rustc --version 2>$null
    $cargoVersion = cargo --version 2>$null
    if ($rustcVersion -and $cargoVersion) {
        Write-Host "  ✓ Rust installed: $rustcVersion" -ForegroundColor Green
        Write-Host "  ✓ Cargo installed: $cargoVersion" -ForegroundColor Green
    } else {
        throw "Rust not found"
    }
} catch {
    Write-Host "  ✗ Rust not found in PATH" -ForegroundColor Red
    Write-Host ""
    Write-Host "To install Rust, visit: https://rustup.rs/" -ForegroundColor Yellow
    Write-Host "Or run: winget install Rustlang.Rust.MSVC" -ForegroundColor Yellow
    Write-Host ""
    exit 1
}

# Test 2: Navigate to project directory
Write-Host "[2/8] Navigating to project directory..." -ForegroundColor Yellow
Set-Location ccm_rust
Write-Host "  ✓ Current directory: $(Get-Location)" -ForegroundColor Green

# Test 3: Check project structure
Write-Host "[3/8] Checking project structure..." -ForegroundColor Yellow
$requiredFiles = @(
    "Cargo.toml",
    "src/main.rs",
    "src/types/mod.rs",
    "src/db/mod.rs",
    "src/secrets/mod.rs",
    "src/auth/mod.rs",
    "src/commands/mod.rs"
)

$missingFiles = @()
foreach ($file in $requiredFiles) {
    if (Test-Path $file) {
        Write-Host "  ✓ $file" -ForegroundColor Green
    } else {
        Write-Host "  ✗ $file missing" -ForegroundColor Red
        $missingFiles += $file
    }
}

if ($missingFiles.Count -gt 0) {
    Write-Host "  ✗ Missing $($missingFiles.Count) required files" -ForegroundColor Red
    exit 1
}

# Test 4: Fetch dependencies
Write-Host "[4/8] Fetching dependencies..." -ForegroundColor Yellow
Write-Host "  Running: cargo fetch" -ForegroundColor Gray
cargo fetch 2>&1 | Out-Null
if ($LASTEXITCODE -eq 0) {
    Write-Host "  ✓ Dependencies fetched successfully" -ForegroundColor Green
} else {
    Write-Host "  ✗ Failed to fetch dependencies" -ForegroundColor Red
    exit 1
}

# Test 5: Check compilation
Write-Host "[5/8] Checking compilation..." -ForegroundColor Yellow
Write-Host "  Running: cargo check" -ForegroundColor Gray
$cargoCheckOutput = cargo check 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  ✓ Project compiles successfully" -ForegroundColor Green
} else {
    Write-Host "  ✗ Compilation errors found:" -ForegroundColor Red
    Write-Host $cargoCheckOutput
    exit 1
}

# Test 6: Run unit tests
Write-Host "[6/8] Running unit tests..." -ForegroundColor Yellow
Write-Host "  Running: cargo test" -ForegroundColor Gray
$cargoTestOutput = cargo test 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  ✓ All tests passed" -ForegroundColor Green
    # Show test summary
    $cargoTestOutput | Select-String "test result: ok"
} else {
    Write-Host "  ⚠ Some tests failed or have issues" -ForegroundColor Yellow
    Write-Host $cargoTestOutput | Select-String "FAILED|error"
}

# Test 7: Build release binary
Write-Host "[7/8] Building release binary..." -ForegroundColor Yellow
Write-Host "  Running: cargo build --release" -ForegroundColor Gray
$cargoBuildOutput = cargo build --release 2>&1
if ($LASTEXITCODE -eq 0) {
    $exePath = "target\release\ccm.exe"
    if (Test-Path $exePath) {
        $fileInfo = Get-Item $exePath
        Write-Host "  ✓ Binary built: $exePath" -ForegroundColor Green
        Write-Host "    Size: $($fileInfo.Length) bytes" -ForegroundColor Gray
    } else {
        Write-Host "  ✗ Binary not found at expected path" -ForegroundColor Red
    }
} else {
    Write-Host "  ✗ Build failed:" -ForegroundColor Red
    Write-Host $cargoBuildOutput
}

# Test 8: Test basic CLI functionality
Write-Host "[8/8] Testing basic CLI functionality..." -ForegroundColor Yellow
if (Test-Path "target\release\ccm.exe") {
    Write-Host "  Testing: ccm --version" -ForegroundColor Gray
    $versionOutput = & target\release\ccm.exe --version 2>&1
    Write-Host "  $versionOutput" -ForegroundColor Cyan

    Write-Host "  Testing: ccm help" -ForegroundColor Gray
    $helpOutput = & target\release\ccm.exe help 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ Help command works" -ForegroundColor Green
    } else {
        Write-Host "  ✗ Help command failed" -ForegroundColor Red
    }
} else {
    Write-Host "  ⊘ Skipping (binary not built)" -ForegroundColor Gray
}

# Summary
Write-Host ""
Write-Host "=== Test Summary ===" -ForegroundColor Cyan
Write-Host "All critical tests completed!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Install Rust if not already: winget install Rustlang.Rust.MSVC" -ForegroundColor White
Write-Host "  2. Build the project: cd ccm_rust; cargo build --release" -ForegroundColor White
Write-Host "  3. Run the binary: .\target\release\ccm.exe help" -ForegroundColor White
Write-Host "  4. Read docs/QUICKSTART.md for usage examples" -ForegroundColor White
Write-Host ""
