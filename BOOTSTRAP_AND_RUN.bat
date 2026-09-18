@echo off
setlocal EnableExtensions EnableDelayedExpansion
title Modern 3D Creature Evolution - Bootstrap and Run

set "REPO_URL=https://github.com/Quest79/Modern-3D-Creature-Evolution.git"
set "REPO_NAME=Modern-3D-Creature-Evolution"
set "SCRIPT_DIR=%~dp0"

echo.
echo ============================================================
echo   Modern 3D Creature Evolution - Bootstrap and Run
echo ============================================================
echo.

rem ------------------------------------------------------------
rem 1) Make sure WinGet exists
rem ------------------------------------------------------------
where winget >nul 2>&1
if errorlevel 1 (
    echo [ERROR] WinGet is required but was not found.
    echo Install "App Installer" from the Microsoft Store, then run this file again.
    pause
    exit /b 1
)

rem ------------------------------------------------------------
rem 2) Install Git if needed
rem ------------------------------------------------------------
where git >nul 2>&1
if errorlevel 1 (
    echo [SETUP] Git was not found. Installing Git...
    winget install --id Git.Git -e --source winget --accept-package-agreements --accept-source-agreements
    if errorlevel 1 (
        echo [ERROR] Git installation failed.
        pause
        exit /b 1
    )

    rem Refresh the common Git locations into this process PATH.
    set "PATH=%ProgramFiles%\Git\cmd;%LOCALAPPDATA%\Programs\Git\cmd;%PATH%"
)

where git >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Git was installed but is not visible yet.
    echo Close this window and run the batch file again.
    pause
    exit /b 1
)

rem ------------------------------------------------------------
rem 3) Install Rustup / Rust if needed
rem ------------------------------------------------------------
where rustup >nul 2>&1
if errorlevel 1 (
    echo [SETUP] Rust was not found. Installing Rustup...
    winget install --id Rustlang.Rustup -e --source winget --accept-package-agreements --accept-source-agreements
    if errorlevel 1 (
        echo [ERROR] Rustup installation failed.
        pause
        exit /b 1
    )

    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)

where rustup >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Rustup was installed but is not visible yet.
    echo Close this window and run the batch file again.
    pause
    exit /b 1
)

echo [SETUP] Making sure the stable Rust toolchain is installed...
rustup default stable
if errorlevel 1 (
    echo [ERROR] Rust stable toolchain setup failed.
    pause
    exit /b 1
)

set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

rem ------------------------------------------------------------
rem 4) Make sure the MSVC C++ linker/build tools exist
rem ------------------------------------------------------------
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
set "VC_TOOLS_FOUND="

if exist "%VSWHERE%" (
    for /f "usebackq tokens=*" %%I in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do (
        set "VC_TOOLS_FOUND=%%I"
    )
)

if not defined VC_TOOLS_FOUND (
    echo.
    echo [SETUP] Microsoft C++ Build Tools were not detected.
    echo [SETUP] Installing the Visual Studio 2022 C++ Build Tools.
    echo [SETUP] Windows may ask for Administrator permission.
    echo.
    winget install --id Microsoft.VisualStudio.2022.BuildTools -e --source winget ^
        --accept-package-agreements --accept-source-agreements ^
        --override "--wait --passive --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

    if errorlevel 1 (
        echo [ERROR] Visual Studio Build Tools installation failed.
        pause
        exit /b 1
    )
)

rem ------------------------------------------------------------
rem 5) Find, clone, or update the project
rem ------------------------------------------------------------
if exist "%SCRIPT_DIR%.git\" (
    set "PROJECT_DIR=%SCRIPT_DIR%"
) else if exist "%SCRIPT_DIR%%REPO_NAME%\.git\" (
    set "PROJECT_DIR=%SCRIPT_DIR%%REPO_NAME%"
) else (
    echo.
    echo [SETUP] Cloning the project...
    git clone "%REPO_URL%" "%SCRIPT_DIR%%REPO_NAME%"
    if errorlevel 1 (
        echo [ERROR] Git clone failed.
        pause
        exit /b 1
    )
    set "PROJECT_DIR=%SCRIPT_DIR%%REPO_NAME%"
)

cd /d "%PROJECT_DIR%"
if errorlevel 1 (
    echo [ERROR] Could not enter the project directory:
    echo         %PROJECT_DIR%
    pause
    exit /b 1
)

echo.
echo [SETUP] Updating the local checkout...
git fetch origin main
git checkout main >nul 2>&1
git pull --ff-only origin main
if errorlevel 1 (
    echo.
    echo [WARNING] The automatic Git update could not fast-forward.
    echo Your local files were NOT overwritten. Continuing with the local copy.
    echo.
)

rem ------------------------------------------------------------
rem 6) Build the Step 1 executable
rem ------------------------------------------------------------
echo.
echo [BUILD] Building release version...
cargo build --release -p evolab-cli
if errorlevel 1 (
    echo.
    echo [ERROR] Build failed.
    echo If the C++ Build Tools were just installed, reboot Windows once,
    echo then run this batch file again.
    pause
    exit /b 1
)

rem ------------------------------------------------------------
rem 7) Run
rem ------------------------------------------------------------
echo.
echo [RUN] Build complete.
echo.

if "%~1"=="" (
    echo Running the default Step 1 test:
    echo   1000 independent physics worlds, automatic CPU thread count
    echo.
    "%PROJECT_DIR%\target\release\evolab.exe" probe --batch 1000 --workers 0
) else (
    echo Running with custom arguments:
    echo   %*
    echo.
    "%PROJECT_DIR%\target\release\evolab.exe" %*
)

echo.
echo ============================================================
echo   Finished
echo ============================================================
echo.
pause
