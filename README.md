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
  <img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License" />
</p>

Lightweight and elegant Apple Music Discord Rich Presence. Real-time playback, HD album art, and native performance built in pure Rust.

> [!NOTE]
> **Platform Support**: Currently supported and tested on **Windows (10 & 11 64-bit)**.

---

## ✨ Features

- 🎧 **Native Rich Presence**: Displays track title, artist, album, live timestamps & playback progress on Discord.
- 🖼️ **HD Album Artwork**: Automatically fetches high-resolution album covers via Apple iTunes Search API.
- 🔗 **Listen on Apple Music Button**: Clickable Discord button directly leading to the track.
- ⚡ **Zero Bloat & High Performance**: Pure native Rust binary with no heavy web runtimes (uses under 15 MB RAM and ~0% CPU).
- 🎛️ **Interactive Settings GUI**: Lightweight native configuration window displaying current version, adjusting refresh interval, and managing settings.
- 🚀 **Auto-Start**: Option to automatically start MusicPresence when booting your computer.
- 🔄 **GitHub Self-Updater**: Automatic update checks with direct in-app download and installation from GitHub releases.
- 🔴 **System Tray Integration**: Background execution with dynamic hover tooltip and quick context menu.

---

## 📥 Downloads

Binary builds are automatically generated and packaged by GitHub Actions CI/CD:

| Version | Branch | Status | Direct Link |
| :--- | :--- | :--- | :--- |
| **Stable Release** | `main` | Production-ready | [**Download Latest Stable**](https://github.com/crobinaud/musicpresence/releases/latest) |
| **Beta Pre-release** | `develop` | Latest features & test builds | [**Download Latest Beta**](https://github.com/crobinaud/musicpresence/releases) |

---

## 📁 Codebase Architecture

For a detailed technical overview and diagrams, see [**ARCHITECTURE.md**](ARCHITECTURE.md).

```text
src/
├── main.rs          # Main event loop & lifecycle management
├── config.rs        # Configuration loader & manager (config.toml)
├── autostart.rs     # System boot autostart manager
├── updater.rs       # GitHub Release API checker & in-place self-updater
├── gui.rs           # Native interactive Settings GUI window
├── tray.rs          # System tray icon & native context menu
├── itunes.rs        # High-resolution album artwork search via Apple iTunes API
├── media.rs         # Media session & playback capture (GSMTC)
└── discord_ipc.rs   # Low-overhead Discord IPC named pipe protocol
```

---

## 🔴 System Tray & Interactive Settings

The application runs seamlessly in the system tray / notification area:
- **Hover**: Displays the currently playing track name and artist.
- **Left / Right Click**: Opens the context menu:
  - **Music Presence**: Opens the native interactive **Settings** window to view version, adjust refresh interval, toggle startup on boot, and check for updates directly.
  - *Separator*
  - **Quit**: Closes the application and immediately clears your Discord Rich Presence.

---

## 🚀 Getting Started

### Running
1. Download either the **Stable** or **Beta** archive from the download buttons above.
2. Extract the files to your desired folder.
3. Launch **`MusicPresence.exe`**.
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
