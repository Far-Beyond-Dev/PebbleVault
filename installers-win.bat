REM Written By Haywoodspartan (Stephen Hawking)
@echo off
setlocal

REM Define the base paths for Microsoft Build Tools and Windows Kits installations
set "VS_BASE_PATH=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC"
set "WIN_KITS_PATH=C:\Program Files (x86)\Windows Kits\10\Include"

REM Initialize the variables to store the latest versions
set "LATEST_VS_VERSION="
set "LATEST_WIN_KIT_VERSION="

REM Find the latest Visual Studio Build Tools version
for /D %%V in ("%VS_BASE_PATH%\*") do (
    set "VERSION=%%~nxV"
    
    REM Compare the current version with the latest version
    if "%LATEST_VS_VERSION%"=="" (
        set "LATEST_VS_VERSION=%%~nxV"
    ) else (
        for /F "tokens=1-4 delims=. " %%A in ("%LATEST_VS_VERSION%") do (
            for /F "tokens=1-4 delims=. " %%B in ("%%~nxV") do (
                if %%A LSS %%B (
                    set "LATEST_VS_VERSION=%%~nxV"
                ) else if %%A EQU %%B (
                    if %%C LSS %%D (
                        set "LATEST_VS_VERSION=%%~nxV"
                    ) else if %%C EQU %%D (
                        if %%E LSS %%F (
                            set "LATEST_VS_VERSION=%%~nxV"
                        )
                    )
                )
            )
        )
    )
)

REM Check if we found a valid Visual Studio version
if "%LATEST_VS_VERSION%"=="" (
    echo No Visual Studio Build Tools installation found.
    exit /b 1
)

REM Find the latest Windows Kits version that starts with 10.0
for /D %%K in ("%WIN_KITS_PATH%\10.0.*") do (
    set "KIT_VERSION=%%~nxK"
    
    REM Compare the current version with the latest version
    if "%LATEST_WIN_KIT_VERSION%"=="" (
        set "LATEST_WIN_KIT_VERSION=%%~nxK"
    ) else (
        for /F "tokens=1-4 delims=. " %%X in ("%LATEST_WIN_KIT_VERSION%") do (
            for /F "tokens=1-4 delims=. " %%Y in ("%%~nxK") do (
                if %%X LSS %%Y (
                    set "LATEST_WIN_KIT_VERSION=%%~nxK"
                ) else if %%X EQU %%Y (
                    if %%Z LSS %%A (
                        set "LATEST_WIN_KIT_VERSION=%%~nxK"
                    )
                )
            )
        )
    )
)

REM Check if we found a valid Windows Kits version
if "%LATEST_WIN_KIT_VERSION%"=="" (
    echo No valid Windows Kits installation found with version 10.0.
    exit /b 1
)

REM Set paths for cl.exe and lib.exe
set "CL_PATH=%VS_BASE_PATH%\%LATEST_VS_VERSION%\bin\Hostx64\x64"
set "LIB_PATH=%VS_BASE_PATH%\%LATEST_VS_VERSION%\bin\Hostx64\x64"

REM Check for existence of cl.exe and lib.exe
if not exist "%CL_PATH%\cl.exe" (
    echo cl.exe not found in "%CL_PATH%".
    exit /b 1
)

if not exist "%LIB_PATH%\lib.exe" (
    echo lib.exe not found in "%LIB_PATH%".
    exit /b 1
)

REM Set environment variables for the latest versions
set "INCLUDE=%VS_BASE_PATH%\%LATEST_VS_VERSION%\include;%WIN_KITS_PATH%\%LATEST_WIN_KIT_VERSION%\ucrt;%WIN_KITS_PATH%\%LATEST_WIN_KIT_VERSION%\shared;%WIN_KITS_PATH%\%LATEST_WIN_KIT_VERSION%\um"
set "LIB=%VS_BASE_PATH%\%LATEST_VS_VERSION%\lib\x64;C:\Program Files (x86)\Windows Kits\10\Lib\%LATEST_WIN_KIT_VERSION%\ucrt\x64;C:\Program Files (x86)\Windows Kits\10\Lib\%LATEST_WIN_KIT_VERSION%\um\x64"
set "PATH=%CL_PATH%;%LIB_PATH%;%PATH%"

echo Using Visual Studio Build Tools version %LATEST_VS_VERSION%
echo Using Windows Kits version %LATEST_WIN_KIT_VERSION%
echo Environment variables have been set.

cd ./sqlite-amalgamation/

REM Compile sqlite3.c to create an object file
cl.exe /c /O2 /I"%VS_BASE_PATH%\%LATEST_VS_VERSION%\include" /I"%WIN_KITS_PATH%\%LATEST_WIN_KIT_VERSION%\ucrt" sqlite3.c

REM Check if the object file was created
if exist sqlite3.obj (
    REM Create the static library from the object file
    lib.exe /OUT:sqlite3.lib sqlite3.obj
    echo Static library sqlite3.lib created successfully.
) else (
    echo Failed to compile sqlite3.c to an object file.
)

endlocal
pause