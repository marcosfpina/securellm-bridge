use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[derive(Debug, Serialize, Deserialize)]
pub struct WyomingEvent {
    pub r#type: String,
    pub data: Option<Value>,
}

pub struct PiperClient {
    host: String,
    port: u16,
}

impl PiperClient {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
        }
    }

    pub async fn synthesize(&self, text: &str, voice: Option<&str>) -> Result<Vec<f32>> {
        let mut stream = TcpStream::connect(format!("{}:{}", self.host, self.port))
            .await
            .context("Failed to connect to Piper (Wyoming)")?;

        let synth_event = serde_json::json!({
            "type": "synthesize",
            "data": {
                "text": text,
                "voice": {
                    "name": voice.unwrap_or("en_US-lessac-medium")
                }
            }
        });

        let mut event_str = synth_event.to_string();
        event_str.push('\n');
        stream.write_all(event_str.as_bytes()).await?;

        let mut reader = BufReader::new(stream);
        let mut audio_buffer = Vec::new();

        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            if line.is_empty() {
                break;
            }

            let event: WyomingEvent = serde_json::from_str(&line)?;

            match event.r#type.as_str() {
                "audio-chunk" => {
                    if let Some(data) = event.data {
                        let length = data["length"].as_u64().unwrap_or(0) as usize;
                        let mut chunk = vec![0u8; length];
                        reader.read_exact(&mut chunk).await?;

                        for i in (0..length).step_by(2) {
                            if i + 1 < length {
                                let sample = i16::from_le_bytes([chunk[i], chunk[i + 1]]);
                                audio_buffer.push(sample as f32 / 32768.0);
                            }
                        }
                    }
                }
                "audio-stop" => {
                    break;
                }
                _ => {}
            }
        }

        Ok(audio_buffer)
    }
}
