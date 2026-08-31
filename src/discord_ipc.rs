use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

static NONCE_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Serialize, Clone)]
pub struct Timestamps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct Assets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_text: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Button {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct Activity {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub activity_type: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamps: Option<Timestamps>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<Assets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<Button>>,
}

#[derive(Debug, Serialize)]
struct SetActivityArgs {
    pid: u32,
    activity: Option<Activity>,
}

#[derive(Debug, Serialize)]
struct SetActivityPayload {
    cmd: &'static str,
    args: SetActivityArgs,
    nonce: String,
}

#[derive(Debug, Serialize)]
struct HandshakePayload<'a> {
    v: u32,
    client_id: &'a str,
}

pub struct DiscordIpc {
    client_id: String,
    pipe: Option<File>,
    pub is_connected: bool,
}

impl DiscordIpc {
    pub fn new(client_id: String) -> Self {
        let mut ipc = Self {
            client_id,
            pipe: None,
            is_connected: false,
        };
        let _ = ipc.connect();
        ipc
    }

    pub fn update_client_id(&mut self, new_client_id: &str) {
        if self.client_id != new_client_id {
            let _ = self.clear_activity();
            self.client_id = new_client_id.to_string();
            self.pipe = None;
            self.is_connected = false;
            let _ = self.connect();
        }
    }

    /// Attempts to connect to one of the Discord IPC named pipes (0 to 9)
    pub fn connect(&mut self) -> Result<(), String> {
        if self.is_connected && self.pipe.is_some() {
            return Ok(());
        }

        self.pipe = None;
        self.is_connected = false;

        for i in 0..10 {
            let pipe_name = format!(r"\\.\pipe\discord-ipc-{}", i);
            if let Ok(mut file) = OpenOptions::new().read(true).write(true).open(&pipe_name) {
                // Handshake (Opcode 0)
                let handshake = HandshakePayload {
                    v: 1,
                    client_id: &self.client_id,
                };
                let payload = match serde_json::to_string(&handshake) {
                    Ok(p) => p,
                    Err(e) => return Err(e.to_string()),
                };

                if Self::write_frame(&mut file, 0, &payload).is_ok() {
                    // Read handshake response
                    if let Ok((opcode, resp)) = Self::read_frame(&mut file) {
                        if opcode == 1 && resp.contains("READY") {
                            self.pipe = Some(file);
                            self.is_connected = true;
                            return Ok(());
                        }
                    }
                }
            }
        }

        Err("Discord is not running or the IPC pipe is unreachable.".to_string())
    }

    /// Updates the Discord Rich Presence activity
    pub fn set_activity(&mut self, activity: Activity) -> Result<(), String> {
        if !self.is_connected || self.pipe.is_none() {
            self.connect()?;
        }

        let nonce = NONCE_COUNTER.fetch_add(1, Ordering::SeqCst).to_string();
        let payload = SetActivityPayload {
            cmd: "SET_ACTIVITY",
            args: SetActivityArgs {
                pid: process::id(),
                activity: Some(activity),
            },
            nonce,
        };

        let json_str = serde_json::to_string(&payload).map_err(|e| e.to_string())?;

        let res = if let Some(pipe) = self.pipe.as_mut() {
            if Self::write_frame(pipe, 1, &json_str).is_ok() {
                // Drain frame response to prevent buffer overflow
                let _ = Self::read_frame(pipe);
                Ok(())
            } else {
                Err("Error writing to Discord pipe".to_string())
            }
        } else {
            Err("Pipe not connected".to_string())
        };

        if res.is_err() {
            self.is_connected = false;
            self.pipe = None;
        }

        res
    }

    /// Clears the Rich Presence activity
    pub fn clear_activity(&mut self) -> Result<(), String> {
        if !self.is_connected || self.pipe.is_none() {
            return Ok(());
        }

        let nonce = NONCE_COUNTER.fetch_add(1, Ordering::SeqCst).to_string();
        let payload = SetActivityPayload {
            cmd: "SET_ACTIVITY",
            args: SetActivityArgs {
                pid: process::id(),
                activity: None,
            },
            nonce,
        };

        let json_str = serde_json::to_string(&payload).map_err(|e| e.to_string())?;

        let res = if let Some(pipe) = self.pipe.as_mut() {
            if Self::write_frame(pipe, 1, &json_str).is_ok() {
                let _ = Self::read_frame(pipe);
                Ok(())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };

        if res.is_err() {
            self.is_connected = false;
            self.pipe = None;
        }

        res
    }

    fn write_frame(file: &mut File, opcode: u32, payload: &str) -> std::io::Result<()> {
        let payload_bytes = payload.as_bytes();
        let length = payload_bytes.len() as u32;

        let mut header = [0u8; 8];
        header[0..4].copy_from_slice(&opcode.to_le_bytes());
        header[4..8].copy_from_slice(&length.to_le_bytes());

        file.write_all(&header)?;
        file.write_all(payload_bytes)?;
        file.flush()?;
        Ok(())
    }

    fn read_frame(file: &mut File) -> std::io::Result<(u32, String)> {
        let mut header = [0u8; 8];
        file.read_exact(&mut header)?;

        let opcode = u32::from_le_bytes(header[0..4].try_into().unwrap());
        let length = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;

        let mut payload = vec![0u8; length];
        file.read_exact(&mut payload)?;

        let json_str = String::from_utf8_lossy(&payload).into_owned();
        Ok((opcode, json_str))
    }
}

impl Drop for DiscordIpc {
    fn drop(&mut self) {
        let _ = self.clear_activity();
    }
}
