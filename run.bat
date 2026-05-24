@echo off
setlocal

cd /d "%~dp0"

REM Get the version number from Cargo.toml
for /f "tokens=2 delims==" %%A in ('findstr "^version =" Cargo.toml') do (
    set raw_version=%%A
)
REM Remove quotes and spaces
set version=%raw_version:"=%
set version=%version: =%

REM Define output directory
set output_dir=dist
if not exist "%output_dir%" mkdir "%output_dir%"

REM Step 1: Compile the Rust backend in release mode
echo ============================================
echo  Building Rust backend in release mode...
echo ============================================
cargo build --release

if %ERRORLEVEL% neq 0 (
    echo Error: Rust backend build failed.
    exit /b 1
)
echo Rust backend built successfully.

REM Step 2: Package the Python UI with Nuitka
echo ============================================
echo  Packaging Python UI with Nuitka...
echo ============================================
python -m nuitka --standalone --onefile ^
    --enable-plugin=tk-inter ^
    --include-data-file="target/release/backend.exe=target/release/backend.exe" ^
    --include-data-dir="assets=assets" ^
    --output-filename="TwinCAN_v%version%" ^
    --output-dir="%output_dir%" ^
    --remove-output ^
    ui/ui.py

if %ERRORLEVEL% neq 0 (
    echo Error: Nuitka packaging failed.
    exit /b 1
)

echo ============================================
echo  Build complete!
echo  Output: %output_dir%\TwinCAN_v%version%.exe
echo ============================================
