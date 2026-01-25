//! Audio capture using cpal

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct AudioCapture {
    buffer: Arc<Mutex<Vec<f32>>>,
    is_active: Arc<Mutex<bool>>,
    stream: Option<cpal::Stream>,
}

impl AudioCapture {
    pub fn new() -> Result<Self> {
        Ok(Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            is_active: Arc::new(Mutex::new(false)),
            stream: None,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("No input device available")?;

        let config = device
            .default_input_config()
            .context("Failed to get default input config")?;

        let buffer = Arc::clone(&self.buffer);
        let is_active = Arc::clone(&self.is_active);

        // Start recording
        *is_active.lock().unwrap() = true;
        buffer.lock().unwrap().clear();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => self.build_input_stream::<f32>(&device, &config.into(), buffer)?,
            cpal::SampleFormat::I16 => self.build_input_stream::<i16>(&device, &config.into(), buffer)?,
            cpal::SampleFormat::U16 => self.build_input_stream::<u16>(&device, &config.into(), buffer)?,
            _ => anyhow::bail!("Unsupported sample format"),
        };

        stream.play()?;
        self.stream = Some(stream);

        Ok(())
    }

    fn build_input_stream<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        buffer: Arc<Mutex<Vec<f32>>>,
    ) -> Result<cpal::Stream>
    where
        T: cpal::Sample,
    {
        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let mut buf = buffer.lock().unwrap();
                for &sample in data {
                    buf.push(sample.to_f32());
                }
            },
            |err| eprintln!("Audio capture error: {}", err),
            None,
        )?;

        Ok(stream)
    }

    pub fn stop(&mut self) -> Result<Vec<f32>> {
        *self.is_active.lock().unwrap() = false;
        
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }

        let buffer = self.buffer.lock().unwrap();
        Ok(buffer.clone())
    }

    pub fn is_active(&self) -> bool {
        *self.is_active.lock().unwrap()
    }
}
