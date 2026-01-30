@echo off
REM CCM Export Test Script for Windows CMD
REM This script tests exporting functionality with different options

echo === CCM Export Testing Script ===
echo.

REM Test 1: Check Rust Installation
echo [1/10] Checking Rust installation...
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
echo [2/10] Navigating to project directory...
cd ccm_rust
if %ERRORLEVEL% NEQ 0 (
    echo   X Cannot find ccm_rust directory
    pause
    exit /b 1
)
echo   √ Current directory: %CD%
echo.

REM Test 3: Build the project
echo [3/10] Building CCM project...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo   X Build failed
    pause
    exit /b 1
)
echo   √ Build successful
echo.

REM Test 4: Setup test environment
echo [4/10] Setting up test environment...

REM Clean any existing test database and export files
if exist "%USERPROFILE%\.ccm\test_export.db" del "%USERPROFILE%\.ccm\test_export.db" /Q
echo   √ Cleaned existing test database

REM Create test export directory
if exist "%TEMP%\ccm_export_test" rmdir "%TEMP%\ccm_export_test" /S /Q
mkdir "%TEMP%\ccm_export_test"
echo   √ Created test export directory: %TEMP%\ccm_export_test

REM Set environment variable for test database
set CCM_DB_PATH=%USERPROFILE%\.ccm\test_export.db
echo   √ Test database path: %CCM_DB_PATH%
echo.

REM Test 5: Initialize CCM with test PIN
echo [5/10] Initializing CCM with test PIN...
echo 1234 | .\target\release\ccm.exe init --yes
if %ERRORLEVEL% NEQ 0 (
    echo   X Initialization failed
    pause
    exit /b 1
)
echo   √ CCM initialized successfully
echo.

REM Test 6: Import test data
echo [6/10] Importing test data...
.\target\release\ccm.exe import tests\fixtures\json\all-entry-types.json --yes
if %ERRORLEVEL% NEQ 0 (
    echo   X Test data import failed
    pause
    exit /b 1
)
echo   √ Test data imported successfully
echo   Imported from: tests\fixtures\json\all-entry-types.json
echo.

REM Test 7: Export all entries (encrypted)
echo [7/10] Testing export - All entries (encrypted)...
.\target\release\ccm.exe export --output %TEMP%\ccm_export_test\full-backup-encrypted.json
if %ERRORLEVEL% EQU 0 (
    for %%A in (%TEMP%\ccm_export_test\full-backup-encrypted.json) do set FILE_SIZE=%%~zA
    echo   √ Encrypted export successful
    echo   Output file: %TEMP%\ccm_export_test\full-backup-encrypted.json (%FILE_SIZE% bytes)
) else (
    echo   X Encrypted export failed
)
echo.

REM Test 8: Export all entries (decrypted)
echo [8/10] Testing export - All entries (decrypted)...
.\target\release\ccm.exe export --decrypt --output %TEMP%\ccm_export_test\full-backup-decrypted.json
if %ERRORLEVEL% EQU 0 (
    for %%A in (%TEMP%\ccm_export_test\full-backup-decrypted.json) do set FILE_SIZE=%%~zA
    echo   √ Decrypted export successful
    echo   Output file: %TEMP%\ccm_export_test\full-backup-decrypted.json (%FILE_SIZE% bytes)
) else (
    echo   X Decrypted export failed
)
echo.

REM Test 9: Export specific types
echo [9/10] Testing export - Specific types...
set TYPES=api password ssh secret
for %%t in (%TYPES%) do (
    echo Testing %%t type export...
    .\target\release\ccm.exe export --type %%t --decrypt --output %TEMP%\ccm_export_test\%%t-entries.json
    if %ERRORLEVEL% EQU 0 (
        for %%A in (%TEMP%\ccm_export_test\%%t-entries.json) do set FILE_SIZE=%%~zA
        echo   √ %%t export successful
        echo   Output file: %TEMP%\ccm_export_test\%%t-entries.json (%FILE_SIZE% bytes)
    ) else (
        echo   X %%t export failed
    )
)
echo.

REM Test 10: Verify exports and test round-trip import
echo [10/10] Testing export verification and round-trip import...

if exist "%TEMP%\ccm_export_test\full-backup-decrypted.json" (
    REM Clean current database
    if exist "%CCM_DB_PATH%" del "%CCM_DB_PATH%" /Q

    REM Re-initialize
    echo 1234 | .\target\release\ccm.exe init --yes
    if %ERRORLEVEL% NEQ 0 (
        echo   X Re-initialization failed
    ) else (
        REM Import the exported file
        .\target\release\ccm.exe import %TEMP%\ccm_export_test\full-backup-decrypted.json --yes
        if %ERRORLEVEL% EQU 0 (
            echo   √ Round-trip import successful
            echo   Verified export integrity
        ) else (
            echo   X Round-trip import failed
        )
    )
) else (
    echo   X Cannot test round-trip - decrypted export file not found
)
echo.

REM Show summary
echo === Export Test Summary ===
echo.

REM List exported files
echo Exported files:
for %%f in (%TEMP%\ccm_export_test\*) do (
    for %%A in (%%f) do echo   %%~nA%%~xA (%%~zA bytes)
)
echo.

REM Count current entries
echo Checking current entries...
.\target\release\ccm.exe list > temp_list.txt 2>&1
if %ERRORLEVEL% EQU 0 (
    findstr /R "^[[:space:]]*[0-9][0-9]*\." temp_list.txt | find /C "." > temp_count.txt
    set /p ENTRY_COUNT=<temp_count.txt
    echo √ Current entries: %ENTRY_COUNT%
) else (
    echo X Failed to list entries
)
del temp_list.txt temp_count.txt 2>nul
echo.

echo Export testing completed!
echo Test database: %CCM_DB_PATH%
echo Export directory: %TEMP%\ccm_export_test
echo.

REM Clean up
echo Cleaning up test environment...
if exist "%CCM_DB_PATH%" del "%CCM_DB_PATH%" /Q
if exist "%TEMP%\ccm_export_test" rmdir "%TEMP%\ccm_export_test" /S /Q
echo √ Test cleanup completed

pause