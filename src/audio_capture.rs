use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig, SupportedStreamConfig};
use crossbeam_channel::{Sender, bounded};
use anyhow::{Context, Result};
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Debug, Clone)]
pub enum AudioSource {
    Microphone,
    Speaker,
}

pub struct AudioCapture {
    host: Host,
    mic_device: Option<Device>,
    speaker_device: Option<Device>,
}

impl AudioCapture {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        
        Ok(Self {
            host,
            mic_device: None,
            speaker_device: None,
        })
    }

    pub fn list_devices(&self) -> Result<()> {
        tracing::info!("Available audio devices:");
        
        tracing::info!("\n=== Input Devices ===");
        for device in self.host.input_devices()? {
            let name = device.name()?;
            tracing::info!("  - {}", name);
        }
        
        tracing::info!("\n=== Output Devices ===");
        for device in self.host.output_devices()? {
            let name = device.name()?;
            tracing::info!("  - {}", name);
        }
        
        Ok(())
    }

    pub fn init_microphone(&mut self) -> Result<()> {
        self.mic_device = Some(
            self.host
                .default_input_device()
                .context("No input device available")?,
        );
        tracing::info!("Microphone initialized: {}", self.mic_device.as_ref().unwrap().name()?);
        Ok(())
    }

    pub fn init_speaker(&mut self) -> Result<()> {
        // 在 Windows 上，我们需要捕获 loopback 设备
        #[cfg(target_os = "windows")]
        {
            // 尝试找到 loopback 设备
            let devices: Vec<_> = self.host.input_devices()?.collect();
            
            for device in devices {
                let name = device.name().unwrap_or_default();
                // 查找立体声混音或类似设备
                if name.contains("立体声混音") 
                    || name.contains("Stereo Mix") 
                    || name.contains("What U Hear")
                    || name.contains("Wave Out Mix") {
                    self.speaker_device = Some(device);
                    tracing::info!("Speaker loopback device found: {}", name);
                    return Ok(());
                }
            }
            
            tracing::warn!("No loopback device found. Please enable 'Stereo Mix' in Windows sound settings.");
            tracing::warn!("Using default output device as fallback (may not work).");
        }
        
        self.speaker_device = Some(
            self.host
                .default_output_device()
                .context("No output device available")?,
        );
        
        Ok(())
    }

    pub fn create_stream(
        &self,
        source: AudioSource,
        sample_rate: u32,
    ) -> Result<(Stream, crossbeam_channel::Receiver<Vec<f32>>)> {
        let device = match source {
            AudioSource::Microphone => self.mic_device.as_ref()
                .context("Microphone not initialized")?,
            AudioSource::Speaker => self.speaker_device.as_ref()
                .context("Speaker not initialized")?,
        };

        let config = device.default_input_config()?;
        tracing::info!(
            "Device config: {} channels, {} Hz, {:?}",
            config.channels(),
            config.sample_rate().0,
            config.sample_format()
        );

        let (tx, rx) = bounded(1000);
        
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => self.build_stream::<f32>(device, &config.into(), tx)?,
            cpal::SampleFormat::I16 => self.build_stream::<i16>(device, &config.into(), tx)?,
            cpal::SampleFormat::U16 => self.build_stream::<u16>(device, &config.into(), tx)?,
            format => anyhow::bail!("Unsupported sample format: {:?}", format),
        };

        Ok((stream, rx))
    }

    fn build_stream<T>(
        &self,
        device: &Device,
        config: &StreamConfig,
        tx: Sender<Vec<f32>>,
    ) -> Result<Stream>
    where
        T: cpal::Sample + cpal::SizedSample,
        f32: cpal::FromSample<T>,
    {
        let channels = config.channels as usize;
        let err_fn = |err| tracing::error!("Stream error: {}", err);

        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let samples: Vec<f32> = data
                    .iter()
                    .map(|&sample| cpal::Sample::from_sample(sample))
                    .collect();

                // 将多声道转换为单声道（混合）
                let mono: Vec<f32> = samples
                    .chunks(channels)
                    .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                    .collect();

                let _ = tx.try_send(mono);
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }
}
