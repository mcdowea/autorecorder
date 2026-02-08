// MP3编码器模块
// 将WAV音频数据编码为MP3格式

use anyhow::{Context, Result};
use std::path::Path;
use std::fs::File;
use std::io::Write;

pub struct Mp3Encoder {
    sample_rate: u32,
    bit_rate: u32,
    quality: u8,
}

impl Mp3Encoder {
    pub fn new(sample_rate: u32, bit_rate: u32, quality: u8) -> Self {
        Self {
            sample_rate,
            bit_rate,
            quality: quality.min(9),
        }
    }

    /// 将f32样本数据编码为MP3文件
    pub fn encode_to_file<P: AsRef<Path>>(
        &self,
        samples: &[f32],
        output_path: P,
    ) -> Result<()> {
        use mp3lame_encoder::{Builder, FlushNoGap, InterleavedPcm};

        // 创建LAME编码器
        let mut encoder = Builder::new()
            .context("Failed to create LAME encoder")?
            .set_sample_rate(self.sample_rate)
            .context("Failed to set sample rate")?
            .set_num_channels(1)  // 单声道
            .context("Failed to set channels")?
            .set_brate(mp3lame_encoder::Bitrate::Kbps(self.bit_rate as u32))
            .context("Failed to set bitrate")?
            .set_quality(mp3lame_encoder::Quality::Best)
            .context("Failed to set quality")?
            .build()
            .context("Failed to build encoder")?;

        // 转换f32到i16
        let pcm_samples: Vec<i16> = samples
            .iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
            .collect();

        // 编码
        let mut mp3_buffer = Vec::new();
        let input = InterleavedPcm(&pcm_samples);
        
        let encoded = encoder.encode(input)
            .context("Failed to encode audio")?;
        mp3_buffer.extend_from_slice(encoded);

        // Flush剩余数据
        let flushed = encoder.flush::<FlushNoGap>()
            .context("Failed to flush encoder")?;
        mp3_buffer.extend_from_slice(flushed);

        // 写入文件
        let mut file = File::create(output_path)?;
        file.write_all(&mp3_buffer)?;

        Ok(())
    }

    /// 实时编码流式数据
    pub fn create_streaming_encoder(&self) -> Result<StreamingMp3Encoder> {
        StreamingMp3Encoder::new(self.sample_rate, self.bit_rate, self.quality)
    }
}

/// 流式MP3编码器,支持增量编码
pub struct StreamingMp3Encoder {
    encoder: mp3lame_encoder::Encoder,
    mp3_buffer: Vec<u8>,
}

impl StreamingMp3Encoder {
    pub fn new(sample_rate: u32, bit_rate: u32, _quality: u8) -> Result<Self> {
        use mp3lame_encoder::Builder;

        let encoder = Builder::new()
            .context("Failed to create LAME encoder")?
            .set_sample_rate(sample_rate)
            .context("Failed to set sample rate")?
            .set_num_channels(1)
            .context("Failed to set channels")?
            .set_brate(mp3lame_encoder::Bitrate::Kbps(bit_rate as u32))
            .context("Failed to set bitrate")?
            .set_quality(mp3lame_encoder::Quality::Best)
            .context("Failed to set quality")?
            .build()
            .context("Failed to build encoder")?;

        Ok(Self {
            encoder,
            mp3_buffer: Vec::new(),
        })
    }

    /// 编码一批f32样本
    pub fn encode_samples(&mut self, samples: &[f32]) -> Result<()> {
        use mp3lame_encoder::InterleavedPcm;

        // 转换f32到i16
        let pcm_samples: Vec<i16> = samples
            .iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
            .collect();

        let input = InterleavedPcm(&pcm_samples);
        let encoded = self.encoder.encode(input)
            .context("Failed to encode samples")?;
        
        self.mp3_buffer.extend_from_slice(encoded);

        Ok(())
    }

    /// 完成编码并获取所有MP3数据
    pub fn finish(mut self) -> Result<Vec<u8>> {
        use mp3lame_encoder::FlushNoGap;

        let flushed = self.encoder.flush::<FlushNoGap>()
            .context("Failed to flush encoder")?;
        self.mp3_buffer.extend_from_slice(flushed);

        Ok(self.mp3_buffer)
    }

    /// 保存到文件
    pub fn save_to_file<P: AsRef<Path>>(self, path: P) -> Result<()> {
        let data = self.finish()?;
        let mut file = File::create(path)?;
        file.write_all(&data)?;
        Ok(())
    }
}

// 用于WAV格式的简单编码器(作为备份)
pub struct WavEncoder {
    sample_rate: u32,
}

impl WavEncoder {
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }

    pub fn encode_to_file<P: AsRef<Path>>(
        &self,
        samples: &[f32],
        output_path: P,
    ) -> Result<()> {
        use hound::{WavWriter, WavSpec, SampleFormat};

        let spec = WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(output_path, spec)?;

        for &sample in samples {
            let amplitude = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
            writer.write_sample(amplitude)?;
        }

        writer.finalize()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_encoder() {
        let encoder = WavEncoder::new(44100);
        
        // 生成1秒的440Hz正弦波
        let duration = 1.0;
        let frequency = 440.0;
        let sample_rate = 44100.0;
        
        let samples: Vec<f32> = (0..(duration * sample_rate) as usize)
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * std::f32::consts::PI * frequency * t).sin() * 0.5
            })
            .collect();

        encoder.encode_to_file(&samples, "test_output.wav").unwrap();
    }
}
