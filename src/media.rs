use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub state: PlaybackState,
    pub position_secs: u64,
    pub duration_secs: u64,
    pub app_id: String,
}

pub struct MediaManager {
    manager: Option<GlobalSystemMediaTransportControlsSessionManager>,
}

impl MediaManager {
    pub fn new() -> Self {
        let manager = match GlobalSystemMediaTransportControlsSessionManager::RequestAsync() {
            Ok(op) => match op.get() {
                Ok(mgr) => Some(mgr),
                Err(e) => {
                    eprintln!("⚠️  Unable to initialize Windows Media Session Manager: {e}");
                    None
                }
            },
            Err(e) => {
                eprintln!("⚠️  GSMTC RequestAsync error: {e}");
                None
            }
        };

        Self { manager }
    }

    /// Fetches currently playing media information from Apple Music or active media session
    pub fn get_current_track(&mut self, filter_apple_music: bool) -> Option<TrackInfo> {
        if self.manager.is_none()
            && let Ok(op) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            && let Ok(mgr) = op.get()
        {
            self.manager = Some(mgr);
        }

        let mgr = self.manager.as_ref()?;

        let sessions = match mgr.GetSessions() {
            Ok(s) => s,
            Err(_) => return None,
        };

        let mut target_session: Option<GlobalSystemMediaTransportControlsSession> = None;

        for session in &sessions {
            if let Ok(app_id_hstring) = session.SourceAppUserModelId() {
                let app_id = app_id_hstring.to_string();
                let app_id_lower = app_id.to_lowercase();

                let is_apple_music = app_id_lower.contains("apple")
                    && (app_id_lower.contains("music") || app_id_lower.contains("itunes"));

                if is_apple_music {
                    target_session = Some(session.clone());
                    break;
                }
            }
        }

        if target_session.is_none()
            && !filter_apple_music
            && let Ok(current) = mgr.GetCurrentSession()
        {
            target_session = Some(current);
        }

        let session = target_session?;
        let app_id = session
            .SourceAppUserModelId()
            .map(|s| s.to_string())
            .unwrap_or_else(|_| "Unknown".to_string());

        let media_props = match session.TryGetMediaPropertiesAsync() {
            Ok(op) => match op.get() {
                Ok(props) => props,
                Err(_) => return None,
            },
            Err(_) => return None,
        };

        let title = media_props
            .Title()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let artist = media_props
            .Artist()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let album = media_props
            .AlbumTitle()
            .map(|s| s.to_string())
            .unwrap_or_default();

        if title.trim().is_empty() {
            return None;
        }

        let state = match session.GetPlaybackInfo() {
            Ok(info) => match info.PlaybackStatus() {
                Ok(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing) => {
                    PlaybackState::Playing
                }
                Ok(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused) => {
                    PlaybackState::Paused
                }
                Ok(GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped) => {
                    PlaybackState::Stopped
                }
                _ => PlaybackState::Other,
            },
            Err(_) => PlaybackState::Other,
        };

        let (position_secs, duration_secs) = match session.GetTimelineProperties() {
            Ok(timeline) => {
                let pos = timeline
                    .Position()
                    .map(|ts| (ts.Duration / 10_000_000) as u64)
                    .unwrap_or(0);
                let dur = timeline
                    .EndTime()
                    .map(|ts| (ts.Duration / 10_000_000) as u64)
                    .unwrap_or(0);
                (pos, dur)
            }
            Err(_) => (0, 0),
        };

        Some(TrackInfo {
            title,
            artist,
            album,
            state,
            position_secs,
            duration_secs,
            app_id,
        })
    }
}
