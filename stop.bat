@echo off
title Stop Apple Music Discord Presence
taskkill /F /IM AppleMusicPresence.exe /IM amprust.exe >nul 2>&1
echo Apple Music Discord Presence was successfully stopped.
ping 127.0.0.1 -n 2 >nul
