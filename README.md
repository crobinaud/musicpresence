# 🍎 Apple Music Discord Presence (`AppleMusicPresence.exe`)

Ultra-lightweight, native, and highly optimized **Rust** application that dynamically displays **"Listening to <Track Title>"** on your **Discord** profile using **Discord Rich Presence**.

---

## 📁 Codebase Architecture (100% Native Windows)

```text
src/
├── main.rs          # Main event loop & lifecycle management
├── config.rs        # Configuration loader & validator (config.toml)
├── itunes.rs        # High-resolution album artwork search via Apple iTunes API
├── media.rs         # Native Windows GSMTC media session capture
├── discord_ipc.rs   # Low-overhead Discord IPC named pipe protocol
└── tray.rs          # System tray icon & Win32 native context menu
```

---

## 🔴 System Tray Icon & Context Menu

The application features a dedicated **red Apple Music icon (`♫`)** in the Windows notification area (bottom right near the clock):
- **On hover**: Displays the currently playing track and artist.
- **Left/Right click**: Opens the clean 2-option context menu:
  - **Music Presence**: Opens the configuration editor (`config.toml`) to customize settings.
  - **Quit**: Instantly shuts down the background process and clears your Discord presence.

---

## 🚀 Usage

- **Start**: Double-click on **[`AppleMusicPresence.exe`](file:///c:/Users/Cyprien/Documents/amprust/AppleMusicPresence.exe)** or **[`run.bat`](file:///c:/Users/Cyprien/Documents/amprust/run.bat)**.
- **Stop**: Right-click the red Apple Music tray icon > **Quit** (or double-click [`stop.bat`](file:///c:/Users/Cyprien/Documents/amprust/stop.bat)).
