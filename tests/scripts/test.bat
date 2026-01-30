@echo off
REM CCM Rust Test Script for Windows CMD
REM This script tests the Rust refactoring of CCM

echo === CCM Rust Testing Script ===
echo.

REM Test 1: Check Rust Installation
echo [1/8] Checking Rust installation...
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
echo [2/8] Navigating to project directory...
cd ccm_rust
if %ERRORLEVEL% NEQ 0 (
    echo   X Cannot find ccm_rust directory
    pause
    exit /b 1
)
echo   √ Current directory: %CD%
echo.

REM Test 3: Check project structure
echo [3/8] Checking project structure...
if exist "Cargo.toml" echo   √ Cargo.toml
if exist "src\main.rs" echo   √ src\main.rs
if exist "src\types\mod.rs" echo   √ src\types\mod.rs
if exist "src\db\mod.rs" echo   √ src\db\mod.rs
if exist "src\secrets\mod.rs" echo   √ src\secrets\mod.rs
if exist "src\auth\mod.rs" echo   √ src\auth\mod.rs
if exist "src\commands\mod.rs" echo   √ src\commands\mod.rs
echo.

REM Test 4: Fetch dependencies
echo [4/8] Fetching dependencies...
echo   Running: cargo fetch
cargo fetch >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo   √ Dependencies fetched successfully
) else (
    echo   X Failed to fetch dependencies
    pause
    exit /b 1
)
echo.

REM Test 5: Check compilation
echo [5/8] Checking compilation...
echo   Running: cargo check
cargo check
if %ERRORLEVEL% EQU 0 (
    echo   √ Project compiles successfully
) else (
    echo   X Compilation errors found
    pause
    exit /b 1
)
echo.

REM Test 6: Run unit tests
echo [6/8] Running unit tests...
echo   Running: cargo test
cargo test
if %ERRORLEVEL% EQU 0 (
    echo   √ All tests passed
) else (
    echo   ! Some tests failed or have issues
)
echo.

REM Test 7: Build release binary
echo [7/8] Building release binary...
echo   Running: cargo build --release
cargo build --release
if %ERRORLEVEL% EQU 0 (
    if exist "target\release\ccm.exe" (
        echo   √ Binary built: target\release\ccm.exe
        for %%A in ("target\release\ccm.exe") do echo     Size: %%~zA bytes
    ) else (
        echo   X Binary not found at expected path
    )
) else (
    echo   X Build failed
)
echo.

REM Test 8: Test basic CLI functionality
echo [8/8] Testing basic CLI functionality...
if exist "target\release\ccm.exe" (
    echo   Testing: ccm --version
    target\release\ccm.exe --version
    echo.

    echo   Testing: ccm help
    target\release\ccm.exe help
    if %ERRORLEVEL% EQU 0 (
        echo   √ Help command works
    ) else (
        echo   X Help command failed
    )
) else (
    echo   ⊘ Skipping (binary not built)
)
echo.

REM Summary
echo === Test Summary ===
echo All critical tests completed!
echo.
echo Next steps:
echo   1. Install Rust if not already: winget install Rustlang.Rust.MSVC
echo   2. Build the project: cd ccm_rust ^&^& cargo build --release
echo   3. Run the binary: .\target\release\ccm.exe help
echo   4. Read docs\QUICKSTART.md for usage examples
echo.
pause
