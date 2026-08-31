use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const DEFAULT_CLIENT_ID: &str = "1046420605907501056"; // Apple Music Discord Rich Presence Application

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Discord Application Client ID
    pub client_id: String,
    /// Polling interval in milliseconds (e.g., 1500ms)
    pub poll_interval_ms: u64,
    /// Display presence when music is paused
    pub show_paused: bool,
    /// Automatically fetch HD album art from Apple iTunes Search API
    pub fetch_itunes_art: bool,
    /// Display clickable "Listen on Apple Music" button in Discord
    pub enable_listen_button: bool,
    /// Filter exclusively for Apple Music (or any Windows media player)
    pub filter_apple_music_only: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            client_id: DEFAULT_CLIENT_ID.to_string(),
            poll_interval_ms: 1500,
            show_paused: true,
            fetch_itunes_art: true,
            enable_listen_button: true,
            filter_apple_music_only: true,
        }
    }
}

impl Config {
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => match toml::from_str::<Config>(&content) {
                    Ok(mut config) => {
                        let mut modified = false;
                        if config.client_id == "1134547942733979679" {
                            config.client_id = DEFAULT_CLIENT_ID.to_string();
                            modified = true;
                        }
                        if modified {
                            let _ = fs::write(
                                path,
                                toml::to_string_pretty(&config).unwrap_or_default(),
                            );
                        }
                        return config;
                    }
                    Err(e) => {
                        eprintln!(
                            "⚠️  Error reading config.toml ({}), using default values.",
                            e
                        );
                    }
                },
                Err(e) => {
                    eprintln!(
                        "⚠️  Unable to read config.toml ({}), using default values.",
                        e
                    );
                }
            }
        }

        let default_config = Config::default();
        let default_toml = r#"# Apple Music Discord Presence Configuration (amprust)

# Official Discord Application Client ID for Apple Music Rich Presence
client_id = "1046420605907501056"

# Verification interval in milliseconds (1500 = 1.5 seconds)
poll_interval_ms = 1500

# Show Discord presence when music is paused (true = shows "Paused", false = hides presence)
show_paused = true

# Fetch high-resolution album artwork via Apple iTunes API
fetch_itunes_art = true

# Display clickable "Listen on Apple Music" button in Discord
enable_listen_button = true

# Detect Apple Music only (set to false to support any Windows media player)
filter_apple_music_only = true
"#;

        let _ = fs::write(path, default_toml);
        default_config
    }
}

/// Formats the song title respecting Discord size limits (max 128 UTF-8 bytes)
pub fn format_listening_title(title: &str) -> String {
    let trimmed = title.trim();
    let text = if trimmed.len() < 2 {
        if !trimmed.is_empty() {
            trimmed
        } else {
            "Music"
        }
    } else {
        trimmed
    };

    truncate_utf8(text, 128)
}

/// Safely truncates a string respecting UTF-8 character boundaries
fn truncate_utf8(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}
