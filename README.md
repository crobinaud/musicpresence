# 🎵 MusicPresence (`AppleMusicPresence.exe`)

<p align="center">
  <a href="https://github.com/crobinaud/musicpresence/releases/latest">
    <img src="https://img.shields.io/badge/Download_Stable-Windows_x64_(main)-0078D4?style=for-the-badge&logo=windows&logoColor=white" alt="Download Stable" />
  </a>
  &nbsp;&nbsp;
  <a href="https://github.com/crobinaud/musicpresence/releases">
    <img src="https://img.shields.io/badge/Download_Beta-Windows_x64_(develop)-F58220?style=for-the-badge&logo=windows&logoColor=white" alt="Download Beta" />
  </a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Platform-Windows_Only-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Platform: Windows Only" />
  <img src="https://img.shields.io/badge/Language-Rust-dea584?style=flat-square&logo=rust&logoColor=white" alt="Language: Rust" />
  <img src="https://img.shields.io/badge/CI%2FCD-Automated%20Releases-2ea44f?style=flat-square&logo=githubactions&logoColor=white" alt="CI/CD" />
  <img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License" />
</p>

Ultra-lightweight, native, and highly optimized **Rust** application that dynamically displays **"Listening to <Track Title>"** on your **Discord** profile using **Discord Rich Presence** for Apple Music.

> [!NOTE]
> **Platform Availability**: Currently, **MusicPresence** is exclusively available for **Windows (Windows 10 & 11 64-bit)**.

---

## ✨ Features

- 🎧 **Native Rich Presence**: Displays track title, artist, album, live timestamps & playback progress on Discord.
- 🖼️ **HD Album Artwork**: Automatically fetches high-resolution album covers via Apple iTunes Search API.
- 🔗 **Listen on Apple Music Button**: Clickable Discord button directly leading to the track.
- ⚡ **Zero Bloat & High Performance**: Pure native Rust binary with no heavy web runtimes (uses under 15 MB RAM and ~0% CPU).
- 🎛️ **Interactive Settings GUI**: Custom Win32 configuration window displaying current version, adjusting refresh interval, and managing settings.
- 🚀 **Windows Auto-Start**: Explicit option to automatically start MusicPresence when booting Windows.
- 🔄 **GitHub Self-Updater**: Automatic update checks with direct in-app download and installation from GitHub releases.
- 🔴 **System Tray Integration**: Background execution with dynamic hover tooltip and quick context menu.

---

## 📥 Downloads

Binary builds are automatically generated and packaged by GitHub Actions CI/CD:

| Version | Branch | Status | Direct Link |
| :--- | :--- | :--- | :--- |
| **Stable Release** | `main` | Production-ready | [**Download Latest Stable (`.zip` / `.exe`)**](https://github.com/crobinaud/musicpresence/releases/latest) |
| **Beta Pre-release** | `develop` | Latest features & test builds | [**Download Latest Beta**](https://github.com/crobinaud/musicpresence/releases) |

---

## 📁 Codebase Architecture (100% Native Windows)

```text
src/
├── main.rs          # Main event loop & lifecycle management
├── config.rs        # Configuration loader & manager (config.toml)
├── autostart.rs     # Windows Registry HKCU Run manager (startup on boot)
├── updater.rs       # GitHub Release API checker & in-place self-updater
├── gui.rs           # Win32 interactive Settings GUI window
├── tray.rs          # System tray icon & Win32 native context menu
├── itunes.rs        # High-resolution album artwork search via Apple iTunes API
├── media.rs         # Native Windows GSMTC media session capture
└── discord_ipc.rs   # Low-overhead Discord IPC named pipe protocol
```

---

## 🔴 System Tray & Interactive Settings

The application runs seamlessly in the Windows notification area (System Tray):
- **Hover**: Displays the currently playing track name and artist.
- **Left / Right Click**: Opens the context menu:
  - **Music Presence**: Opens the native interactive **Settings** window to view version, adjust refresh interval, toggle Windows startup, and check for updates directly.
  - *Separator*
  - **Quit**: Closes the application and immediately clears your Discord Rich Presence.

---

## 🚀 Getting Started

### Running
1. Download either the **Stable** or **Beta** archive from the download buttons above.
2. Extract the files to your desired folder.
3. Launch **`AppleMusicPresence.exe`**.
4. Start playing music in Apple Music — your Discord profile will automatically reflect your playback!

### Stopping
- Right-click or left-click the Apple Music icon in the system tray > click **Quit**.

---

## 🛠️ Building from Source

```bash
# Clone the repository
git clone https://github.com/crobinaud/musicpresence.git
cd musicpresence

# Build optimized release binary
cargo build --release
```

---

## 📄 License

This project is open source and available under the [MIT License](LICENSE).
