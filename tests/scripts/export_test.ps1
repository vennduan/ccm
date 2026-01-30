# CCM Export Test Script for PowerShell
# This script tests exporting functionality with different options

$ErrorActionPreference = "Continue"
$ProgressPreference = "SilentlyContinue"

Write-Host "=== CCM Export Testing Script ===" -ForegroundColor Cyan
Write-Host ""

# Test 1: Check Rust Installation and Build
Write-Host "[1/10] Checking Rust installation and building project..." -ForegroundColor Yellow
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

# Navigate to project directory
Write-Host "[2/10] Navigating to project directory..." -ForegroundColor Yellow
try {
    Set-Location ccm_rust
    Write-Host "  ✓ Current directory: $(Get-Location)" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Failed to navigate to ccm_rust directory" -ForegroundColor Red
    exit 1
}

# Build the project
Write-Host "[3/10] Building CCM project..." -ForegroundColor Yellow
try {
    $buildOutput = cargo build --release 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ Build successful" -ForegroundColor Green
    } else {
        Write-Host "  ✗ Build failed" -ForegroundColor Red
        Write-Host "  Build output: $buildOutput" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "  ✗ Build command failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Test 4: Setup test environment
Write-Host "[4/10] Setting up test environment..." -ForegroundColor Yellow

# Clean any existing test database and export files
$testDbPath = "$env:USERPROFILE\.ccm\test_export.db"
$testExportDir = "$env:TEMP\ccm_export_test"

if (Test-Path $testDbPath) {
    Remove-Item $testDbPath -Force -ErrorAction SilentlyContinue
    Write-Host "  ✓ Cleaned existing test database" -ForegroundColor Green
}

if (Test-Path $testExportDir) {
    Remove-Item $testExportDir -Recurse -Force -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Path $testExportDir -Force | Out-Null
Write-Host "  ✓ Created test export directory: $testExportDir" -ForegroundColor Green

# Set environment variable for test database
$env:CCM_DB_PATH = $testDbPath
Write-Host "  ✓ Test database path: $testDbPath" -ForegroundColor Green

# Test 5: Initialize CCM with test PIN
Write-Host "[5/10] Initializing CCM with test PIN..." -ForegroundColor Yellow
$initCommand = "echo '1234' | .\target\release\ccm.exe init --yes"
try {
    $initOutput = Invoke-Expression $initCommand 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ CCM initialized successfully" -ForegroundColor Green
    } else {
        Write-Host "  ✗ Initialization failed" -ForegroundColor Red
        Write-Host "  Output: $initOutput" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "  ✗ Initialization command failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Test 6: Import test data
Write-Host "[6/10] Importing test data..." -ForegroundColor Yellow
$testDataPath = "tests\fixtures\json\all-entry-types.json"
try {
    $importOutput = .\target\release\ccm.exe import $testDataPath --yes 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ Test data imported successfully" -ForegroundColor Green
        Write-Host "  Imported from: $testDataPath" -ForegroundColor Gray
    } else {
        Write-Host "  ✗ Test data import failed" -ForegroundColor Red
        Write-Host "  Output: $importOutput" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "  ✗ Test data import command failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Test 7: Export all entries (encrypted)
Write-Host "[7/10] Testing export - All entries (encrypted)..." -ForegroundColor Yellow
$encryptedExportPath = "$testExportDir\full-backup-encrypted.json"
try {
    $exportOutput = .\target\release\ccm.exe export --output $encryptedExportPath 2>&1
    if ($LASTEXITCODE -eq 0 -and (Test-Path $encryptedExportPath)) {
        $fileSize = (Get-Item $encryptedExportPath).Length
        Write-Host "  ✓ Encrypted export successful" -ForegroundColor Green
        Write-Host "  Output file: $encryptedExportPath ($fileSize bytes)" -ForegroundColor Gray
    } else {
        Write-Host "  ✗ Encrypted export failed" -ForegroundColor Red
        Write-Host "  Output: $exportOutput" -ForegroundColor Red
    }
} catch {
    Write-Host "  ✗ Encrypted export command failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 8: Export all entries (decrypted)
Write-Host "[8/10] Testing export - All entries (decrypted)..." -ForegroundColor Yellow
$decryptedExportPath = "$testExportDir\full-backup-decrypted.json"
try {
    $exportOutput = .\target\release\ccm.exe export --decrypt --output $decryptedExportPath 2>&1
    if ($LASTEXITCODE -eq 0 -and (Test-Path $decryptedExportPath)) {
        $fileSize = (Get-Item $decryptedExportPath).Length
        Write-Host "  ✓ Decrypted export successful" -ForegroundColor Green
        Write-Host "  Output file: $decryptedExportPath ($fileSize bytes)" -ForegroundColor Gray
    } else {
        Write-Host "  ✗ Decrypted export failed" -ForegroundColor Red
        Write-Host "  Output: $exportOutput" -ForegroundColor Red
    }
} catch {
    Write-Host "  ✗ Decrypted export command failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 9: Export specific types
Write-Host "[9/10] Testing export - Specific types..." -ForegroundColor Yellow
$types = @("api", "password", "ssh", "secret")
foreach ($type in $types) {
    $typeExportPath = "$testExportDir\$type-entries.json"
    try {
        $exportOutput = .\target\release\ccm.exe export --type $type --decrypt --output $typeExportPath 2>&1
        if ($LASTEXITCODE -eq 0 -and (Test-Path $typeExportPath)) {
            $fileSize = (Get-Item $typeExportPath).Length
            Write-Host "  ✓ $type export successful" -ForegroundColor Green
            Write-Host "  Output file: $typeExportPath ($fileSize bytes)" -ForegroundColor Gray
        } else {
            Write-Host "  ✗ $type export failed" -ForegroundColor Red
            Write-Host "  Output: $exportOutput" -ForegroundColor Red
        }
    } catch {
        Write-Host "  ✗ $type export command failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Test 10: Verify exports and test round-trip import
Write-Host "[10/10] Testing export verification and round-trip import..." -ForegroundColor Yellow

# Test round-trip with decrypted export
if (Test-Path $decryptedExportPath) {
    try {
        # Clean current database
        Remove-Item $testDbPath -Force -ErrorAction SilentlyContinue

        # Re-initialize
        $initCommand = "echo '1234' | .\target\release\ccm.exe init --yes"
        $initOutput = Invoke-Expression $initCommand 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Host "  ✗ Re-initialization failed" -ForegroundColor Red
        }

        # Import the exported file
        $roundTripOutput = .\target\release\ccm.exe import $decryptedExportPath --yes 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  ✓ Round-trip import successful" -ForegroundColor Green
            Write-Host "  Verified export integrity" -ForegroundColor Gray
        } else {
            Write-Host "  ✗ Round-trip import failed" -ForegroundColor Red
            Write-Host "  Output: $roundTripOutput" -ForegroundColor Red
        }
    } catch {
        Write-Host "  ✗ Round-trip test failed: $($_.Exception.Message)" -ForegroundColor Red
    }
} else {
    Write-Host "  ✗ Cannot test round-trip - decrypted export file not found" -ForegroundColor Red
}

# Show summary
Write-Host ""
Write-Host "=== Export Test Summary ===" -ForegroundColor Cyan

# List exported files
Write-Host "Exported files:" -ForegroundColor Yellow
Get-ChildItem $testExportDir -File | ForEach-Object {
    $fileSize = $_.Length
    Write-Host "  $($_.Name) ($fileSize bytes)" -ForegroundColor Gray
}

# Count current entries
Write-Host "Checking current entries..." -ForegroundColor Yellow
try {
    $listOutput = .\target\release\ccm.exe list 2>&1
    if ($LASTEXITCODE -eq 0) {
        $entryCount = ($listOutput | Select-String -Pattern "^\s*\d+\." | Measure-Object).Count
        Write-Host "✓ Current entries: $entryCount" -ForegroundColor Green
    } else {
        Write-Host "✗ Failed to list entries" -ForegroundColor Red
        Write-Host "Output: $listOutput" -ForegroundColor Red
    }
} catch {
    Write-Host "✗ List command failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "Export testing completed!" -ForegroundColor Green
Write-Host "Test database: $testDbPath" -ForegroundColor Gray
Write-Host "Export directory: $testExportDir" -ForegroundColor Gray
Write-Host ""

# Clean up
Write-Host "Cleaning up test environment..." -ForegroundColor Yellow
Remove-Item $testDbPath -Force -ErrorAction SilentlyContinue
Remove-Item $testExportDir -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item Env:\CCM_DB_PATH -ErrorAction SilentlyContinue
Write-Host "✓ Test cleanup completed" -ForegroundColor Green