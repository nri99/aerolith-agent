@echo off
set "BIN=%~dp0target\release\aerolith-agent.exe"
if not exist "%BIN%" set "BIN=%~dp0target\x86_64-pc-windows-gnu\release\aerolith-agent.exe"
if exist "%BIN%" (
    start "" "%BIN%" %*
) else (
    echo [ERROR] Aerolith Agent binary not found.
    echo Please run: cargo build --release
    pause
)
