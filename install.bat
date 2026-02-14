@echo off
:: Nes Installer — one click install
:: Builds from source (needs Rust) or uses existing binary

setlocal

set "INSTALL_DIR=%USERPROFILE%\.nes"
set "BIN=%INSTALL_DIR%\nes.exe"

echo.
echo  nes - the nestea shell installer
echo  =================================
echo.

:: Check for existing binary in repo
if exist "%~dp0nes.exe" (
    echo [1/3] Found pre-built nes.exe
    goto :install
)

:: Check for cargo
where cargo >nul 2>&1
if errorlevel 1 (
    echo ERROR: Rust is not installed.
    echo Install Rust from: https://rustup.rs
    echo Then run this installer again.
    pause
    exit /b 1
)

echo [1/3] Building nes...
cargo build --release --manifest-path "%~dp0Cargo.toml"
if errorlevel 1 (
    echo Build failed. Make sure Rust is installed correctly.
    pause
    exit /b 1
)

:: Find the built binary
if exist "%~dp0target\release\nes.exe" (
    copy /y "%~dp0target\release\nes.exe" "%~dp0nes.exe" >nul
) else (
    for /r "%~dp0target" %%f in (nes.exe) do (
        if exist "%%f" (
            copy /y "%%f" "%~dp0nes.exe" >nul
            goto :install
        )
    )
    echo ERROR: Could not find built binary.
    pause
    exit /b 1
)

:install
echo [2/3] Installing to %INSTALL_DIR%\

if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"
copy /y "%~dp0nes.exe" "%BIN%" >nul

:: Add to PATH if not already there
echo %PATH% | findstr /i /c:"%INSTALL_DIR%" >nul
if errorlevel 1 (
    echo [3/3] Adding to PATH...
    setx PATH "%PATH%;%INSTALL_DIR%" >nul 2>&1
    if errorlevel 1 (
        :: PATH too long for setx, use PowerShell
        powershell -NoProfile -Command "$p = [Environment]::GetEnvironmentVariable('PATH','User'); if ($p -notlike '*\.nes*') { [Environment]::SetEnvironmentVariable('PATH', $p + ';%INSTALL_DIR%', 'User') }"
    )
    echo    Added %INSTALL_DIR% to your PATH.
    echo    Restart your terminal for PATH to take effect.
) else (
    echo [3/3] PATH already configured.
)

echo.
echo  Done! Nes is installed.
echo  Open a new terminal and type: nes help
echo.
pause
