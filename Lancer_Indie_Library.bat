@echo off
title Indie Library — YeohGames
echo ===================================================
echo     Indie Library - Lanceur Automatique Windows
echo ===================================================
echo.

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [INFO] Cargo/Rust n'a pas ete detecte sur ce PC Windows.
    echo [INFO] Installation automatique de Rust en cours...
    winget install -e --id Rustlang.Rustup --accept-source-agreements --accept-package-agreements
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)

if exist "target\release\indie-library.exe" (
    echo [LANCEMENT] Demarrage de Indie Library...
    start "" "target\release\indie-library.exe"
    exit /b 0
)

echo [COMPILATION] Compilation de l'executable Windows en cours...
cargo run --release
pause