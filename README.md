# 🎵 MusicPresence

<p align="center">
  <a href="https://github.com/crobinaud/musicpresence/releases/latest">
    <img src="https://img.shields.io/badge/Download_Stable-Latest_Release-2ea44f?style=for-the-badge" alt="Download Stable" />
  </a>
  &nbsp;&nbsp;
  <a href="https://github.com/crobinaud/musicpresence/releases">
    <img src="https://img.shields.io/badge/Download_Beta-develop_branch-F58220?style=for-the-badge" alt="Download Beta" />
  </a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Platform-Windows-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Platform: Windows" />
  <img src="https://img.shields.io/badge/Language-Rust-dea584?style=flat-square&logo=rust&logoColor=white" alt="Language: Rust" />
  <img src="https://img.shields.io/badge/CI%2FCD-Automated%20Releases-2ea44f?style=flat-square&logo=githubactions&logoColor=white" alt="CI/CD" />
  <img src="https://img.shields.io/badge/License-Proprietary-lightgrey?style=flat-square" alt="License" />
</p>

Lightweight and elegant Apple Music Discord Rich Presence. Real-time playback, HD album art, and native performance built in pure Rust.

> [!NOTE]
> **Platform Support**: MusicPresence is developed specifically for **Windows (10 & 11 64-bit)** and will remain strictly **Windows-only** (relying on Windows native GSMTC and Win32 APIs).

---

## ✨ Features

- 🎧 **Real-Time Discord Presence**: Displays playback status (Playing / Paused), track title, artist, album, live timestamps, and interactive progress bar via low-latency native IPC.
- 🖼️ **HD Album Artwork**: Automatically queries the Apple iTunes Search API for crisp, high-resolution album covers.
- 🔗 **Customizable Discord Button**: Clickable profile button linking directly to the track on Apple Music with a configurable label (e.g. *"Écouter ensemble"*).
- ⚡ **Ultra-Lightweight & Fast**: Written in 100% pure Rust with zero web/Electron bloat (~15 MB RAM, virtually 0% CPU).
- 🎛️ **Native Settings Window**: Built with native Win32/DWM dark mode to configure polling interval, button label, autostart, and updates.
- 🔴 **System Tray Companion**: Runs silently in the background with hover track info and a quick context menu.
- 🚀 **Windows Autostart**: Optional native Windows registry integration to start automatically on system boot.
- 🔄 **Built-in Self-Updater**: Background update checks with one-click direct upgrade from GitHub Releases.
- 🛡️ **Single-Instance Protection**: Named Windows Mutex prevents multiple background instances from running simultaneously.

---

## 🔮 Planned Support (Roadmap)

While currently focused on Apple Music, upcoming versions will expand player integration on Windows to:
- 🟣 **Deezer**
- 📦 **Amazon Music**
- 🎵 **iTunes** (Windows Desktop)

*(MusicPresence will remain Windows-only across all future player integrations.)*

---

## 🚀 Getting Started

### Installation Options

| Version | Description | Link |
| :--- | :--- | :--- |
| **Installer (.exe)** *(Recommended)* | Setup wizard with Start Menu & Desktop shortcuts, clean Windows integration | [**Download Latest Stable**](https://github.com/crobinaud/musicpresence/releases/latest) |
| **Portable (.zip / .exe)** | Standalone binary with no installation required | [**Releases Archive**](https://github.com/crobinaud/musicpresence/releases) |
| **Beta Pre-release** | Bleeding-edge test builds from the `develop` branch | [**Download Beta**](https://github.com/crobinaud/musicpresence/releases) |

### Controls & Tray Usage

The application lives in your Windows system tray:
- **Hover**: Shows the currently playing track title and artist.
- **Left / Right Click**:
  - **Music Presence**: Opens the native configuration settings window.
  - **Quit**: Exits the application and immediately clears your Discord presence.

### Uninstallation

If installed using `MusicPresence-Setup.exe`, uninstall at any time from **Windows Settings > Apps > Installed apps > MusicPresence > Uninstall**.

---

## 🛠️ Building from Source

```bash
# Clone the repository
git clone https://github.com/crobinaud/musicpresence.git
cd musicpresence

# Build optimized release binary
cargo build --release
```

The compiled binary will be located at `target/release/MusicPresence.exe`.

---

## 📄 License

Copyright © 2026 Cyprien ROBINAUD. All rights reserved.

This software is **Source-Available**: the source code is public for educational review and security auditing, but redistribution, commercial use, and unauthorized forks are strictly prohibited. See [LICENSE](LICENSE) for details.
