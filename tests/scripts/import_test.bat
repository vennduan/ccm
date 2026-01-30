@echo off
REM CCM Import Test Script for Windows CMD
REM This script tests importing CSV and JSON fixture files

echo === CCM Import Testing Script ===
echo.

REM Test 1: Check Rust Installation
echo [1/12] Checking Rust installation...
where rustc >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo   X Rust not found in PATH
    echo.
    echo To install Rust, visit: https://rustup.rs/
    echo Or run: winget install Rustlang.Rust.MSVC
    echo.
    pause
    exit /b 1
)

for /f "tokens=*" %%i in ('rustc --version') do set RUSTC_VERSION=%%i
for /f "tokens=*" %%i in ('cargo --version') do set CARGO_VERSION=%%i
echo   √ Rust installed: %RUSTC_VERSION%
echo   √ Cargo installed: %CARGO_VERSION%
echo.

REM Test 2: Navigate to project directory
echo [2/12] Navigating to project directory...
cd ccm_rust
if %ERRORLEVEL% NEQ 0 (
    echo   X Cannot find ccm_rust directory
    pause
    exit /b 1
)
echo   √ Current directory: %CD%
echo.

REM Test 3: Build the project
echo [3/12] Building CCM project...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo   X Build failed
    pause
    exit /b 1
)
echo   √ Build successful
echo.

REM Test 4: Setup test environment
echo [4/12] Setting up test environment...

REM Clean any existing test database
if exist "%USERPROFILE%\.ccm\test_import.db" del "%USERPROFILE%\.ccm\test_import.db" /Q
echo   √ Cleaned existing test database

REM Set environment variable for test database
set CCM_DB_PATH=%USERPROFILE%\.ccm\test_import.db
echo   √ Test database path: %CCM_DB_PATH%
echo.

REM Test 5: Initialize CCM with test PIN
echo [5/12] Initializing CCM with test PIN...
echo 1234 | .\target\release\ccm.exe init --yes
if %ERRORLEVEL% NEQ 0 (
    echo   X Initialization failed
    pause
    exit /b 1
)
echo   √ CCM initialized successfully
echo.

REM Test 6: Test CSV Import - Generic Format
echo [6/12] Testing CSV import - Generic format...
.\target\release\ccm.exe import tests\fixtures\csv\generic-format.csv --yes
if %ERRORLEVEL% EQU 0 (
    echo   √ Generic CSV import successful
    echo   Imported from: tests\fixtures\csv\generic-format.csv
) else (
    echo   X Generic CSV import failed
)
echo.

REM Test 7: Test CSV Import - Browser Formats
echo [7/12] Testing CSV import - Browser formats...
set BROWSER_FORMATS=chrome-format.csv edge-format.csv firefox-format.csv safari-format.csv
for %%f in (%BROWSER_FORMATS%) do (
    echo Testing %%f...
    .\target\release\ccm.exe import tests\fixtures\csv\%%f --yes
    if %ERRORLEVEL% EQU 0 (
        echo   √ %%f import successful
    ) else (
        echo   X %%f import failed
    )
)
echo.

REM Test 8: Test CSV Import - Special Cases
echo [8/12] Testing CSV import - Special cases...
set SPECIAL_CASES=special-characters.csv unicode-utf8.csv edge-cases.csv windows-crlf.csv
for %%f in (%SPECIAL_CASES%) do (
    echo Testing %%f...
    .\target\release\ccm.exe import tests\fixtures\csv\%%f --yes
    if %ERRORLEVEL% EQU 0 (
        echo   √ %%f import successful
    ) else (
        echo   X %%f import failed
    )
)
echo.

REM Test 9: Test JSON Import - Minimal Backup
echo [9/12] Testing JSON import - Minimal backup...
.\target\release\ccm.exe import tests\fixtures\json\minimal-backup.json --yes
if %ERRORLEVEL% EQU 0 (
    echo   √ Minimal JSON backup import successful
    echo   Imported from: tests\fixtures\json\minimal-backup.json
) else (
    echo   X Minimal JSON backup import failed
)
echo.

REM Test 10: Test JSON Import - Complete Backup
echo [10/12] Testing JSON import - Complete backup...
.\target\release\ccm.exe import tests\fixtures\json\complete-backup.json --yes
if %ERRORLEVEL% EQU 0 (
    echo   √ Complete JSON backup import successful
    echo   Imported from: tests\fixtures\json\complete-backup.json
) else (
    echo   X Complete JSON backup import failed
)
echo.

REM Test 11: Test JSON Import - All Entry Types
echo [11/12] Testing JSON import - All entry types...
.\target\release\ccm.exe import tests\fixtures\json\all-entry-types.json --yes
if %ERRORLEVEL% EQU 0 (
    echo   √ All entry types JSON import successful
    echo   Imported from: tests\fixtures\json\all-entry-types.json
) else (
    echo   X All entry types JSON import failed
)
echo.

REM Test 12: Test JSON Import - Special Characters
echo [12/12] Testing JSON import - Special characters...
.\target\release\ccm.exe import tests\fixtures\json\special-characters.json --yes
if %ERRORLEVEL% EQU 0 (
    echo   √ Special characters JSON import successful
    echo   Imported from: tests\fixtures\json\special-characters.json
) else (
    echo   X Special characters JSON import failed
)
echo.

REM Show summary
echo === Import Test Summary ===
echo.

REM Count entries
echo Checking imported entries...
.\target\release\ccm.exe list > temp_list.txt 2>&1
if %ERRORLEVEL% EQU 0 (
    findstr /R "^[[:space:]]*[0-9][0-9]*\." temp_list.txt | find /C "." > temp_count.txt
    set /p ENTRY_COUNT=<temp_count.txt
    echo √ Total entries imported: %ENTRY_COUNT%
) else (
    echo X Failed to list entries
)
del temp_list.txt temp_count.txt 2>nul
echo.

echo Import testing completed!
echo Test database: %CCM_DB_PATH%
echo.

REM Clean up
echo Cleaning up test environment...
if exist "%CCM_DB_PATH%" del "%CCM_DB_PATH%" /Q
echo √ Test cleanup completed

pause