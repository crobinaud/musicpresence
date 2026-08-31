use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ItunesMetadata {
    pub artwork_url: String,
    pub track_url: Option<String>,
}

#[derive(Deserialize)]
struct ItunesResponse {
    results: Vec<ItunesTrack>,
}

#[derive(Deserialize)]
struct ItunesTrack {
    #[serde(rename = "artworkUrl100")]
    artwork_url_100: Option<String>,
    #[serde(rename = "trackViewUrl")]
    track_view_url: Option<String>,
}

pub struct ItunesClient {
    cache: HashMap<String, Option<ItunesMetadata>>,
    agent: ureq::Agent,
}

impl ItunesClient {
    pub fn new() -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(4))
            .user_agent("AppleMusicDiscordRPC/1.0")
            .build();

        Self {
            cache: HashMap::new(),
            agent,
        }
    }

    /// Searches HD metadata for a track with in-memory caching
    pub fn search(&mut self, title: &str, artist: &str) -> Option<ItunesMetadata> {
        let cache_key = format!(
            "{} --- {}",
            title.trim().to_lowercase(),
            artist.trim().to_lowercase()
        );

        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        let metadata = self.fetch_from_api(title, artist);
        self.cache.insert(cache_key, metadata.clone());
        metadata
    }

    fn fetch_from_api(&self, title: &str, artist: &str) -> Option<ItunesMetadata> {
        let query = format!("{} {}", title, artist);
        let encoded_query = encode_query(&query);
        let url = format!(
            "https://itunes.apple.com/search?term={}&media=music&entity=song&limit=1",
            encoded_query
        );

        let response = match self.agent.get(&url).call() {
            Ok(res) => res,
            Err(e) => {
                eprintln!("⚠️  iTunes search skipped for \"{}\": {}", title, e);
                return None;
            }
        };

        let parsed: ItunesResponse = match response.into_json() {
            Ok(json) => json,
            Err(_) => return None,
        };

        let track = parsed.results.into_iter().next()?;

        let artwork_url = track.artwork_url_100.map(|url| {
            // Replace 100x100 thumbnail with 1024x1024 high resolution
            url.replace("100x100bb", "1024x1024bb")
                .replace("100x100", "1024x1024")
        })?;

        Some(ItunesMetadata {
            artwork_url,
            track_url: track.track_view_url,
        })
    }
}

fn encode_query(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 2);
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}
