// MP3编码器模块
// 将WAV音频数据编码为MP3格式

use anyhow::Result;
use std::path::Path;
use std::fs::File;
use std::io::Write;
use std::mem::MaybeUninit;

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
        let mut encoder = Builder::new().expect("Failed to create LAME encoder");
        encoder.set_sample_rate(self.sample_rate).expect("Failed to set sample rate");
        encoder.set_num_channels(1).expect("Failed to set channels");
        
        // 设置比特率
        let bitrate = match self.bit_rate {
            64 => mp3lame_encoder::Bitrate::Kbps64,
            96 => mp3lame_encoder::Bitrate::Kbps96,
            128 => mp3lame_encoder::Bitrate::Kbps128,
            192 => mp3lame_encoder::Bitrate::Kbps192,
            256 => mp3lame_encoder::Bitrate::Kbps256,
            320 => mp3lame_encoder::Bitrate::Kbps320,
            _ => mp3lame_encoder::Bitrate::Kbps128,
        };
        encoder.set_brate(bitrate).expect("Failed to set bitrate");
        
        // 设置质量
        let quality = match self.quality {
            0 => mp3lame_encoder::Quality::Best,
            1..=4 => mp3lame_encoder::Quality::Good,
            5..=7 => mp3lame_encoder::Quality::Good,
            _ => mp3lame_encoder::Quality::Good,
        };
        encoder.set_quality(quality).expect("Failed to set quality");
        
        let mut encoder = encoder.build().expect("Failed to build encoder");

        // 转换f32到i16
        let pcm_samples: Vec<i16> = samples
            .iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
            .collect();

        // 编码
        let mut mp3_buffer = Vec::with_capacity(pcm_samples.len());
        let input = InterleavedPcm(&pcm_samples);
        
        // 为输出分配足够的空间
        let mut output = vec![MaybeUninit::uninit(); pcm_samples.len() * 5 / 4 + 7200];
        let encoded_size = encoder.encode(input, &mut output)
            .expect("Failed to encode audio");
        
        // 安全地转换已初始化的部分
        unsafe {
            mp3_buffer.extend_from_slice(std::slice::from_raw_parts(
                output.as_ptr() as *const u8,
                encoded_size,
            ));
        }

        // Flush剩余数据
        let mut flush_output = vec![MaybeUninit::uninit(); 7200];
        let flushed_size = encoder.flush::<FlushNoGap>(&mut flush_output)
            .expect("Failed to flush encoder");
        
        unsafe {
            mp3_buffer.extend_from_slice(std::slice::from_raw_parts(
                flush_output.as_ptr() as *const u8,
                flushed_size,
            ));
        }

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
    pub fn new(sample_rate: u32, bit_rate: u32, quality: u8) -> Result<Self> {
        use mp3lame_encoder::Builder;

        let mut builder = Builder::new().expect("Failed to create LAME encoder");
        builder.set_sample_rate(sample_rate).expect("Failed to set sample rate");
        builder.set_num_channels(1).expect("Failed to set channels");
        
        // 设置比特率
        let bitrate = match bit_rate {
            64 => mp3lame_encoder::Bitrate::Kbps64,
            96 => mp3lame_encoder::Bitrate::Kbps96,
            128 => mp3lame_encoder::Bitrate::Kbps128,
            192 => mp3lame_encoder::Bitrate::Kbps192,
            256 => mp3lame_encoder::Bitrate::Kbps256,
            320 => mp3lame_encoder::Bitrate::Kbps320,
            _ => mp3lame_encoder::Bitrate::Kbps128,
        };
        builder.set_brate(bitrate).expect("Failed to set bitrate");
        
        // 设置质量
        let qual = match quality {
            0 => mp3lame_encoder::Quality::Best,
            1..=4 => mp3lame_encoder::Quality::Good,
            5..=7 => mp3lame_encoder::Quality::Good,
            _ => mp3lame_encoder::Quality::Good,
        };
        builder.set_quality(qual).expect("Failed to set quality");
        
        let encoder = builder.build().expect("Failed to build encoder");

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
        
        // 为输出分配足够的空间
        let mut output = vec![MaybeUninit::uninit(); pcm_samples.len() * 5 / 4 + 7200];
        let encoded_size = self.encoder.encode(input, &mut output)
            .expect("Failed to encode samples");
        
        // 安全地转换已初始化的部分
        unsafe {
            self.mp3_buffer.extend_from_slice(std::slice::from_raw_parts(
                output.as_ptr() as *const u8,
                encoded_size,
            ));
        }

        Ok(())
    }

    /// 完成编码并获取所有MP3数据
    pub fn finish(mut self) -> Result<Vec<u8>> {
        use mp3lame_encoder::FlushNoGap;

        let mut flush_output = vec![MaybeUninit::uninit(); 7200];
        let flushed_size = self.encoder.flush::<FlushNoGap>(&mut flush_output)
            .expect("Failed to flush encoder");
        
        unsafe {
            self.mp3_buffer.extend_from_slice(std::slice::from_raw_parts(
                flush_output.as_ptr() as *const u8,
                flushed_size,
            ));
        }

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
