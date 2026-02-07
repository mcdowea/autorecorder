use anyhow::{Context, Result};
use hound::{WavWriter, WavSpec, SampleFormat};
use std::path::Path;

pub struct AudioEncoder {
    writer: WavWriter<std::io::BufWriter<std::fs::File>>,
    sample_rate: u32,
}

impl AudioEncoder {
    pub fn new(
        output_path: &Path,
        sample_rate: u32,
        _bit_rate: u32,      // Not used for WAV
        _quality: u8,         // Not used for WAV
    ) -> Result<Self> {
        let spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let writer = WavWriter::create(output_path, spec)
            .with_context(|| format!("Failed to create WAV file: {:?}", output_path))?;

        tracing::info!("Created WAV encoder: {:?}", output_path);
        tracing::info!("Sample rate: {} Hz, Channels: 1, Bits: 16", sample_rate);

        Ok(Self {
            writer,
            sample_rate,
        })
    }

    pub fn encode_samples(&mut self, samples: &[f32]) -> Result<()> {
        if samples.is_empty() {
            return Ok(());
        }

        // Convert f32 samples to i16
        for &sample in samples {
            let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
            self.writer.write_sample(sample_i16)?;
        }

        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        self.writer.finalize()?;
        tracing::info!("WAV encoding finished");
        Ok(())
    }
}
