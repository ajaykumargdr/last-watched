@echo off

REM Unregister the DLL
regsvr32 /u D:\genesis\dll_exp\last-watched\target\debug\last_watched.dll

REM Kill the Explorer process
taskkill /F /IM explorer.exe

REM remove .dll file
del D:\genesis\dll_exp\last-watched\target\debug\last_watched.dll

REM Restart Explorer
start explorer.exe