//! TTS Engine using system text-to-speech

use anyhow::Result;
use tts::Tts;

pub struct TtsEngine {
    tts: Tts,
}

impl TtsEngine {
    pub fn new() -> Result<Self> {
        let tts = Tts::default()?;
        Ok(Self { tts })
    }

    pub async fn speak(&mut self, text: &str) -> Result<()> {
        // TTS crate is synchronous, run in blocking task
        let text = text.to_string();
        let mut tts_clone = self.tts.clone();
        
        tokio::task::spawn_blocking(move || {
            tts_clone.speak(&text, false)
        })
        .await??;
        
        Ok(())
    }

    pub async fn is_speaking(&self) -> bool {
        self.tts.is_speaking().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tts_creation() {
        let engine = TtsEngine::new();
        // May not be available in all environments
        if engine.is_err() {
            eprintln!("TTS backend not available");
        }
    }
}
