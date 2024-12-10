@echo off

REM Unregister the DLL
regsvr32 /u D:\genesis\dll_exp\last-watched\target\debug\last_watched.dll

REM Kill the Explorer process
taskkill /F /IM explorer.exe

REM Build the project three times with Cargo
cargo build
cargo build
cargo build

REM Register the DLL
regsvr32 D:\genesis\dll_exp\last-watched\target\debug\last_watched.dll

REM Restart Explorer
start explorer.exe