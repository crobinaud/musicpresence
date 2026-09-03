use serde::Deserialize;
use std::env;
use std::fs::{self, File};
use std::process::Command;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_REPO: &str = "crobinaud/musicpresence";

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub download_url: Option<String>,
    pub release_url: String,
}

/// Parses a version string like "v1.2.3" or "1.2.3" into (major, minor, patch)
fn parse_version(v: &str) -> (u64, u64, u64) {
    let clean = v.trim_start_matches('v').trim();
    let parts: Vec<&str> = clean.split('.').collect();
    let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts
        .get(2)
        .and_then(|s| s.split('-').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (major, minor, patch)
}

/// Checks if v2 is strictly newer than v1
fn is_newer_version(current: &str, latest: &str) -> bool {
    let curr = parse_version(current);
    let lat = parse_version(latest);
    lat > curr
}

/// Clean up any leftover temporary files from previous updates
pub fn cleanup_old_binary() {
    if let Ok(current_exe) = env::current_exe() {
        let old_exe = current_exe.with_extension("exe.old");
        if old_exe.exists() {
            let _ = fs::remove_file(old_exe);
        }
    }
}

/// Queries GitHub API for the latest release
pub fn check_for_updates() -> Result<Option<UpdateInfo>, String> {
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        GITHUB_REPO
    );

    let resp = match ureq::get(&url)
        .set("User-Agent", "MusicPresence-Updater")
        .set("Accept", "application/vnd.github.v3+json")
        .timeout(std::time::Duration::from_secs(8))
        .call()
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(404, _)) => return Ok(None),
        Err(e) => return Err(format!("Failed to reach GitHub API: {}", e)),
    };

    let release: GithubRelease = resp
        .into_json()
        .map_err(|e| format!("Failed to parse release response: {}", e))?;

    let latest_tag = release.tag_name.trim().to_string();
    let latest_version = latest_tag.trim_start_matches('v').to_string();

    if is_newer_version(CURRENT_VERSION, &latest_version) {
        // Look for .exe asset directly
        let exe_asset = release
            .assets
            .into_iter()
            .find(|a| a.name == "MusicPresence.exe" || a.name.ends_with(".exe"))
            .map(|a| a.browser_download_url);

        Ok(Some(UpdateInfo {
            current_version: CURRENT_VERSION.to_string(),
            latest_version,
            download_url: exe_asset,
            release_url: release.html_url,
        }))
    } else {
        Ok(None)
    }
}

/// Downloads the new executable and replaces the current running process
pub fn apply_update(download_url: &str) -> Result<(), String> {
    let current_exe = env::current_exe().map_err(|e| e.to_string())?;
    let parent_dir = current_exe
        .parent()
        .ok_or_else(|| "Unable to locate application directory.".to_string())?;

    let new_exe = parent_dir.join("MusicPresence.exe.new");
    let old_exe = current_exe.with_extension("exe.old");

    // Download the new binary
    let resp = ureq::get(download_url)
        .set("User-Agent", "MusicPresence-Updater")
        .timeout(std::time::Duration::from_secs(30))
        .call()
        .map_err(|e| format!("Failed to download update: {}", e))?;

    let mut reader = resp.into_reader();
    let mut file =
        File::create(&new_exe).map_err(|e| format!("Failed to create temp file: {}", e))?;
    std::io::copy(&mut reader, &mut file)
        .map_err(|e| format!("Download transfer failed: {}", e))?;
    drop(file);

    // Clean any prior .old file
    if old_exe.exists() {
        let _ = fs::remove_file(&old_exe);
    }

    // Rename current running executable to .old
    fs::rename(&current_exe, &old_exe)
        .map_err(|e| format!("Failed to rename current binary: {}", e))?;

    // Move .new to current executable path
    if let Err(e) = fs::rename(&new_exe, &current_exe) {
        // Rollback if failed
        let _ = fs::rename(&old_exe, &current_exe);
        return Err(format!("Failed to install new binary: {}", e));
    }

    // Launch the updated executable
    let _ = Command::new(&current_exe).spawn();
    std::process::exit(0);
}
