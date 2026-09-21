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

where winget >nul 2>&1
if errorlevel 1 (
    echo [ERROR] WinGet is required but was not found.
    echo Install "App Installer" from the Microsoft Store, then run this file again.
    pause
    exit /b 1
)

where git >nul 2>&1
if errorlevel 1 (
    echo [SETUP] Git was not found. Installing Git...
    winget install --id Git.Git -e --source winget --accept-package-agreements --accept-source-agreements
    if errorlevel 1 (
        echo [ERROR] Git installation failed.
        pause
        exit /b 1
    )
    set "PATH=%ProgramFiles%\Git\cmd;%LOCALAPPDATA%\Programs\Git\cmd;%PATH%"
)

where git >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Git was installed but is not visible yet.
    echo Close this window and run this file again.
    pause
    exit /b 1
)

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
    echo Close this window and run this file again.
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

rem The articulated CUDA solver compiles its GPU kernel with NVRTC at runtime.
rem If this PC has an NVIDIA GPU, make sure the CUDA Toolkit/NVRTC is present.
set "CUDA_NVRTC_FOUND="
if exist "%ProgramFiles%\NVIDIA GPU Computing Toolkit\CUDA" (
    for /d %%D in ("%ProgramFiles%\NVIDIA GPU Computing Toolkit\CUDA\v*") do (
        for %%F in ("%%~fD\bin\nvrtc64_*.dll") do (
            if exist "%%~fF" set "CUDA_NVRTC_FOUND=%%~fF"
        )
    )
)

where nvidia-smi >nul 2>&1
if not errorlevel 1 if not defined CUDA_NVRTC_FOUND (
    echo.
    echo [SETUP] NVIDIA GPU detected. Installing CUDA Toolkit for GPU physics...
    winget install --id Nvidia.CUDA -e --source winget --accept-package-agreements --accept-source-agreements
    if errorlevel 1 (
        echo.
        echo [ERROR] CUDA Toolkit installation failed.
        echo CUDA evolution requires NVRTC from the CUDA Toolkit.
        pause
        exit /b 1
    )
)

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
echo [SETUP] Synchronizing exactly to the latest main branch...
git fetch --prune origin main
if errorlevel 1 (
    echo.
    echo [ERROR] Could not fetch the latest main branch.
    echo The app will not launch from a stale checkout.
    pause
    exit /b 1
)

git checkout main >nul 2>&1
if errorlevel 1 (
    echo.
    echo [ERROR] Could not switch to the main branch.
    echo Resolve local Git changes, then run this file again.
    pause
    exit /b 1
)

git reset --hard origin/main
if errorlevel 1 (
    echo.
    echo [ERROR] Could not synchronize the local checkout to origin/main.
    pause
    exit /b 1
)

for /f "delims=" %%C in ('git rev-parse --short HEAD') do (
    echo [SETUP] Running main at commit %%C
)

echo.
echo [BUILD] Building release backend...
cargo build --release -p evolab-cli
if errorlevel 1 (
    echo.
    echo [ERROR] Build failed.
    echo If the C++ Build Tools were just installed, reboot Windows once,
    echo then run this batch file again.
    pause
    exit /b 1
)

if /i "%~1"=="--cli" (
    shift
    echo.
    echo [RUN] Starting headless CLI...
    "%PROJECT_DIR%\target\release\evolab.exe" %*
    echo.
    pause
    exit /b %errorlevel%
)

call :FindGodot

if not defined GODOT_CMD (
    echo.
    echo [SETUP] Godot 4 was not found. Installing Godot...
    winget install --id GodotEngine.GodotEngine -e --source winget --accept-package-agreements --accept-source-agreements
    if errorlevel 1 (
        echo.
        echo [ERROR] Godot installation failed.
        pause
        exit /b 1
    )

    rem WinGet often says a shell restart is needed. Do not rely on PATH;
    rem search the WinGet package directory directly in this same run.
    call :FindGodot
)

if not defined GODOT_CMD (
    echo.
    echo [ERROR] Godot is installed, but the executable could not be located.
    echo Looked in PATH, WinGet Links, and WinGet Packages.
    echo.
    echo Try closing this window and running this batch file once more.
    pause
    exit /b 1
)

echo.
echo [CHECK] Validating Godot project and GDScript...
"%GODOT_CMD%" --headless --editor --quit --path "%PROJECT_DIR%\apps\godot"
if errorlevel 1 (
    echo.
    echo [ERROR] Godot project validation failed.
    echo Fix the script/scene errors shown above before launching the GUI.
    pause
    exit /b 1
)

echo.
echo [RUN] Launching Modern 3D Creature Evolution GUI...
echo [RUN] Godot: %GODOT_CMD%
echo.
"%GODOT_CMD%" --path "%PROJECT_DIR%\apps\godot"

if errorlevel 1 (
    echo.
    echo [ERROR] Godot exited with an error.
    pause
    exit /b 1
)

endlocal
exit /b 0

:FindGodot
set "GODOT_CMD="

for %%N in (godot.exe godot4.exe godot) do (
    if not defined GODOT_CMD (
        for /f "delims=" %%G in ('where %%N 2^>nul') do (
            if not defined GODOT_CMD set "GODOT_CMD=%%~fG"
        )
    )
)

if not defined GODOT_CMD if exist "%LOCALAPPDATA%\Microsoft\WinGet\Links\godot.exe" (
    set "GODOT_CMD=%LOCALAPPDATA%\Microsoft\WinGet\Links\godot.exe"
)

if not defined GODOT_CMD if exist "%LOCALAPPDATA%\Microsoft\WinGet\Links\godot4.exe" (
    set "GODOT_CMD=%LOCALAPPDATA%\Microsoft\WinGet\Links\godot4.exe"
)

if not defined GODOT_CMD if exist "%LOCALAPPDATA%\Microsoft\WinGet\Packages" (
    for /r "%LOCALAPPDATA%\Microsoft\WinGet\Packages" %%G in (Godot*_win64.exe) do (
        if not defined GODOT_CMD set "GODOT_CMD=%%~fG"
    )
)

if not defined GODOT_CMD if exist "%LOCALAPPDATA%\Microsoft\WinGet\Packages" (
    for /r "%LOCALAPPDATA%\Microsoft\WinGet\Packages" %%G in (godot.exe) do (
        if not defined GODOT_CMD set "GODOT_CMD=%%~fG"
    )
)

exit /b 0
