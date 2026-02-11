@echo off

@REM echo Script dir: %~dp0

net session >nul 2>&1
if %errorlevel% neq 0 (
    @REM echo Elevate:'%~dp0'
    powershell -Command "Start-Process '%~f0' -WorkingDirectory '%~dp0' -Verb RunAs"
    exit /b
)

schtasks /create /tn "Bo\keympostor-startup" /xml "%~dp0\task.xml"
pause