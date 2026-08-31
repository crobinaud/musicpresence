#![windows_subsystem = "windows"]

mod autostart;
mod config;
mod discord_ipc;
mod gui;
mod itunes;
mod media;
mod tray;
mod updater;

use config::Config;
use discord_ipc::{Activity, Assets, Button, DiscordIpc, Timestamps};
use itunes::ItunesClient;
use media::{MediaManager, PlaybackState, TrackInfo};
use std::env;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tray::TrayIcon;

fn main() {
    // Prevent multiple simultaneous instances on Windows
    #[cfg(windows)]
    unsafe {
        use windows::core::w;
        use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
        use windows::Win32::System::Threading::CreateMutexW;

        use windows::Win32::UI::WindowsAndMessaging::SetProcessDPIAware;

        let _ = SetProcessDPIAware();

        let _mutex = CreateMutexW(
            None,
            true,
            w!("AppleMusicDiscordPresence_SingleInstanceMutex"),
        );
        if GetLastError() == ERROR_ALREADY_EXISTS {
            return;
        }
    }

    // Ensure correct working directory when launched via GUI/shortcut
    if let Ok(exe_path) = env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let _ = env::set_current_dir(parent);
        }
    }

    // Clean up any temporary files from past updates
    updater::cleanup_old_binary();

    // Load or create configuration
    let config = Arc::new(RwLock::new(Config::load_or_create("config.toml")));

    // Sync autostart registry state with config if enabled
    if let Ok(cfg) = config.read() {
        if cfg.auto_start && !autostart::is_autostart_enabled() {
            let _ = autostart::set_autostart(true);
        }
    }

    // Check for updates in background if auto-update is enabled
    if config.read().map(|c| c.auto_update).unwrap_or(true) {
        thread::spawn(|| {
            let _ = updater::check_for_updates();
        });
    }

    let initial_client_id = config
        .read()
        .map(|c| c.client_id.clone())
        .unwrap_or_else(|_| config::DEFAULT_CLIENT_ID.to_string());

    // Initialize core modules
    let mut media_manager = MediaManager::new();
    let mut itunes_client = ItunesClient::new();
    let mut discord = DiscordIpc::new(initial_client_id);
    let tray = TrayIcon::new(config.clone());

    let mut last_track: Option<TrackInfo> = None;
    let mut last_display_time = SystemTime::now();

    loop {
        // Check if user clicked Exit/Quit in the System Tray menu
        if tray.is_exit_requested() {
            let _ = discord.clear_activity();
            break;
        }

        let cfg = match config.read() {
            Ok(c) => c.clone(),
            Err(_) => Config::default(),
        };

        discord.update_client_id(&cfg.client_id);

        let current_track = media_manager.get_current_track(cfg.filter_apple_music_only);

        match &current_track {
            Some(track) => {
                let should_update = match &last_track {
                    None => true,
                    Some(prev) => {
                        prev.title != track.title
                            || prev.artist != track.artist
                            || prev.album != track.album
                            || prev.state != track.state
                            || (track.position_secs.abs_diff(prev.position_secs) > 5
                                && track.state == PlaybackState::Playing)
                    }
                };

                if should_update || last_display_time.elapsed().unwrap_or_default().as_secs() >= 10
                {
                    last_display_time = SystemTime::now();

                    // Search for high-resolution album artwork and Apple Music link
                    let itunes_meta = if cfg.fetch_itunes_art {
                        itunes_client.search(&track.title, &track.artist)
                    } else {
                        None
                    };

                    let artwork_url = itunes_meta
                        .as_ref()
                        .map(|m| m.artwork_url.clone())
                        .unwrap_or_else(|| {
                            "https://cdn.rcd.gg/PreMiD/websites/A/Apple%20Music/assets/logo.png"
                                .to_string()
                        });

                    let track_url = itunes_meta.as_ref().and_then(|m| m.track_url.clone());

                    // Build Discord Rich Presence Activity
                    if track.state == PlaybackState::Playing
                        || (track.state == PlaybackState::Paused && cfg.show_paused)
                    {
                        let mut activity = Activity {
                            activity_type: Some(2),
                            ..Default::default()
                        };

                        // Name set to song title
                        let listening_name = config::format_listening_title(&track.title);
                        activity.name = Some(listening_name);

                        // Song title, artist and album details
                        activity.details = Some(track.title.clone());
                        let state_str = if !track.album.is_empty() {
                            format!("{} — {}", track.artist, track.album)
                        } else {
                            track.artist.clone()
                        };
                        activity.state = Some(state_str);

                        // Timestamps for live progress bar on Discord
                        if track.state == PlaybackState::Playing && track.duration_secs > 0 {
                            let now_secs = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            let start = now_secs.saturating_sub(track.position_secs);
                            let end = start + track.duration_secs;
                            activity.timestamps = Some(Timestamps {
                                start: Some(start),
                                end: Some(end),
                            });
                        }

                        // Assets (HD Album Artwork + Play/Pause Badges)
                        let is_playing = track.state == PlaybackState::Playing;
                        activity.assets = Some(Assets {
                            large_image: Some(artwork_url.clone()),
                            large_text: if !track.album.is_empty() {
                                Some(track.album.clone())
                            } else {
                                Some(track.title.clone())
                            },
                            small_image: Some(if is_playing {
                                "play".to_string()
                            } else {
                                "pause".to_string()
                            }),
                            small_text: Some(if is_playing {
                                "Playing".to_string()
                            } else {
                                "Paused".to_string()
                            }),
                        });

                        // "Listen on Apple Music" button
                        if cfg.enable_listen_button {
                            let url =
                                track_url.unwrap_or_else(|| "https://music.apple.com".to_string());
                            activity.buttons = Some(vec![Button {
                                label: "Listen on Apple Music".to_string(),
                                url,
                            }]);
                        }

                        let _ = discord.set_activity(activity);

                        tray.update_status(&format!("{} - {}", track.title, track.artist));
                    } else {
                        // Paused and show_paused is disabled
                        let _ = discord.clear_activity();
                        tray.update_status("Music paused");
                    }
                }
            }
            None => {
                if last_track.is_some() {
                    let _ = discord.clear_activity();
                    tray.update_status("Idle (no playback)");
                }
            }
        }

        last_track = current_track;

        // Periodic polling sliced into 50ms intervals to quickly respond to quit requests
        let poll_ms = cfg.poll_interval_ms;
        let slices = (poll_ms / 50).max(1);
        for _ in 0..slices {
            if tray.is_exit_requested() {
                let _ = discord.clear_activity();
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
    }
}
