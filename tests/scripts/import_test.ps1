# CCM Import Test Script for PowerShell
# This script tests importing CSV and JSON fixture files

$ErrorActionPreference = "Continue"
$ProgressPreference = "SilentlyContinue"

Write-Host "=== CCM Import Testing Script ===" -ForegroundColor Cyan
Write-Host ""

# Test 1: Check Rust Installation and Build
Write-Host "[1/12] Checking Rust installation and building project..." -ForegroundColor Yellow
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
Write-Host "[2/12] Navigating to project directory..." -ForegroundColor Yellow
try {
    Set-Location ccm_rust
    Write-Host "  ✓ Current directory: $(Get-Location)" -ForegroundColor Green
} catch {
    Write-Host "  ✗ Failed to navigate to ccm_rust directory" -ForegroundColor Red
    exit 1
}

# Build the project
Write-Host "[3/12] Building CCM project..." -ForegroundColor Yellow
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
Write-Host "[4/12] Setting up test environment..." -ForegroundColor Yellow

# Clean any existing test database
$testDbPath = "$env:USERPROFILE\.ccm\test_import.db"
if (Test-Path $testDbPath) {
    Remove-Item $testDbPath -Force -ErrorAction SilentlyContinue
    Write-Host "  ✓ Cleaned existing test database" -ForegroundColor Green
}

# Set environment variable for test database
$env:CCM_DB_PATH = $testDbPath
Write-Host "  ✓ Test database path: $testDbPath" -ForegroundColor Green

# Test 5: Initialize CCM with test PIN
Write-Host "[5/12] Initializing CCM with test PIN..." -ForegroundColor Yellow
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

# Test 6: Test CSV Import - Generic Format
Write-Host "[6/12] Testing CSV import - Generic format..." -ForegroundColor Yellow
$csvGenericPath = "tests\fixtures\csv\generic-format.csv"
try {
    $importOutput = .\target\release\ccm.exe import $csvGenericPath --yes 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ Generic CSV import successful" -ForegroundColor Green
        Write-Host "  Imported from: $csvGenericPath" -ForegroundColor Gray
    } else {
        Write-Host "  ✗ Generic CSV import failed" -ForegroundColor Red
        Write-Host "  Output: $importOutput" -ForegroundColor Red
    }
} catch {
    Write-Host "  ✗ Generic CSV import command failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 7: Test CSV Import - Browser Formats
Write-Host "[7/12] Testing CSV import - Browser formats..." -ForegroundColor Yellow
$browserFormats = @("chrome-format.csv", "edge-format.csv", "firefox-format.csv", "safari-format.csv")
foreach ($format in $browserFormats) {
    $csvPath = "tests\fixtures\csv\$format"
    try {
        $importOutput = .\target\release\ccm.exe import $csvPath --yes 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  ✓ $format import successful" -ForegroundColor Green
        } else {
            Write-Host "  ✗ $format import failed" -ForegroundColor Red
            Write-Host "  Output: $importOutput" -ForegroundColor Red
        }
    } catch {
        Write-Host "  ✗ $format import command failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Test 8: Test CSV Import - Special Cases
Write-Host "[8/12] Testing CSV import - Special cases..." -ForegroundColor Yellow
$specialCases = @("special-characters.csv", "unicode-utf8.csv", "edge-cases.csv", "windows-crlf.csv")
foreach ($case in $specialCases) {
    $csvPath = "tests\fixtures\csv\$case"
    try {
        $importOutput = .\target\release\ccm.exe import $csvPath --yes 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  ✓ $case import successful" -ForegroundColor Green
        } else {
            Write-Host "  ✗ $case import failed" -ForegroundColor Red
            Write-Host "  Output: $importOutput" -ForegroundColor Red
        }
    } catch {
        Write-Host "  ✗ $case import command failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Test 9: Test JSON Import - Minimal Backup
Write-Host "[9/12] Testing JSON import - Minimal backup..." -ForegroundColor Yellow
$jsonPath = "tests\fixtures\json\minimal-backup.json"
try {
    $importOutput = .\target\release\ccm.exe import $jsonPath --yes 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ Minimal JSON backup import successful" -ForegroundColor Green
        Write-Host "  Imported from: $jsonPath" -ForegroundColor Gray
    } else {
        Write-Host "  ✗ Minimal JSON backup import failed" -ForegroundColor Red
        Write-Host "  Output: $importOutput" -ForegroundColor Red
    }
} catch {
    Write-Host "  ✗ Minimal JSON backup import command failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 10: Test JSON Import - Complete Backup
Write-Host "[10/12] Testing JSON import - Complete backup..." -ForegroundColor Yellow
$jsonPath = "tests\fixtures\json\complete-backup.json"
try {
    $importOutput = .\target\release\ccm.exe import $jsonPath --yes 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ Complete JSON backup import successful" -ForegroundColor Green
        Write-Host "  Imported from: $jsonPath" -ForegroundColor Gray
    } else {
        Write-Host "  ✗ Complete JSON backup import failed" -ForegroundColor Red
        Write-Host "  Output: $importOutput" -ForegroundColor Red
    }
} catch {
    Write-Host "  ✗ Complete JSON backup import command failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 11: Test JSON Import - All Entry Types
Write-Host "[11/12] Testing JSON import - All entry types..." -ForegroundColor Yellow
$jsonPath = "tests\fixtures\json\all-entry-types.json"
try {
    $importOutput = .\target\release\ccm.exe import $jsonPath --yes 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ All entry types JSON import successful" -ForegroundColor Green
        Write-Host "  Imported from: $jsonPath" -ForegroundColor Gray
    } else {
        Write-Host "  ✗ All entry types JSON import failed" -ForegroundColor Red
        Write-Host "  Output: $importOutput" -ForegroundColor Red
    }
} catch {
    Write-Host "  ✗ All entry types JSON import command failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 12: Test JSON Import - Special Characters
Write-Host "[12/12] Testing JSON import - Special characters..." -ForegroundColor Yellow
$jsonPath = "tests\fixtures\json\special-characters.json"
try {
    $importOutput = .\target\release\ccm.exe import $jsonPath --yes 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ Special characters JSON import successful" -ForegroundColor Green
        Write-Host "  Imported from: $jsonPath" -ForegroundColor Gray
    } else {
        Write-Host "  ✗ Special characters JSON import failed" -ForegroundColor Red
        Write-Host "  Output: $importOutput" -ForegroundColor Red
    }
} catch {
    Write-Host "  ✗ Special characters JSON import command failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Show summary
Write-Host ""
Write-Host "=== Import Test Summary ===" -ForegroundColor Cyan

# Count entries
Write-Host "Checking imported entries..." -ForegroundColor Yellow
try {
    $listOutput = .\target\release\ccm.exe list 2>&1
    if ($LASTEXITCODE -eq 0) {
        $entryCount = ($listOutput | Select-String -Pattern "^\s*\d+\." | Measure-Object).Count
        Write-Host "✓ Total entries imported: $entryCount" -ForegroundColor Green
    } else {
        Write-Host "✗ Failed to list entries" -ForegroundColor Red
        Write-Host "Output: $listOutput" -ForegroundColor Red
    }
} catch {
    Write-Host "✗ List command failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "Import testing completed!" -ForegroundColor Green
Write-Host "Test database: $testDbPath" -ForegroundColor Gray
Write-Host ""

# Clean up
Write-Host "Cleaning up test environment..." -ForegroundColor Yellow
Remove-Item $testDbPath -Force -ErrorAction SilentlyContinue
Remove-Item Env:\CCM_DB_PATH -ErrorAction SilentlyContinue
Write-Host "✓ Test cleanup completed" -ForegroundColor Green