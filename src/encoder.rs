use anyhow::{anyhow, Result};
use mp3lame_encoder::{Builder, FlushNoGap, InterleavedPcm};
use std::path::Path;
use std::fs::File;
use std::io::Write;
use tracing::{info, debug};

pub struct Mp3Encoder {
    sample_rate: u32,
    channels: u16,
    bitrate: u32,
    quality: u8,
}

impl Mp3Encoder {
    pub fn new(sample_rate: u32, channels: u16, bitrate: u32, quality: u8) -> Self {
        Self {
            sample_rate,
            channels,
            bitrate,
            quality: quality.min(9), // LAME quality is 0-9
        }
    }
    
    /// 将 PCM 数据编码为 MP3 并保存到文件
    pub fn encode_to_file<P: AsRef<Path>>(
        &self,
        pcm_data: &[f32],
        output_path: P,
    ) -> Result<()> {
        info!(
            "开始编码 MP3: 采样率 {}, 声道 {}, 比特率 {} kbps",
            self.sample_rate, self.channels, self.bitrate
        );
        
        let mut encoder = Builder::new().ok_or_else(|| anyhow!("无法创建 MP3 编码器"))?;
        
        encoder.set_sample_rate(self.sample_rate)?;
        encoder.set_num_channels(self.channels as u8)?;
        encoder.set_brate(mp3lame_encoder::Birtate::Kbps(self.bitrate as u32))?;
        encoder.set_quality(mp3lame_encoder::Quality::Best)?;
        
        let mut encoder = encoder.build()?;
        
        let mut output_file = File::create(output_path.as_ref())?;
        
        // 转换 f32 到 i16
        let pcm_i16: Vec<i16> = pcm_data
            .iter()
            .map(|&sample| {
                let clamped = sample.max(-1.0).min(1.0);
                (clamped * 32767.0) as i16
            })
            .collect();
        
        // 编码数据
        let mut mp3_buffer = Vec::new();
        let frame_size = 1152 * self.channels as usize; // MP3 帧大小
        
        for chunk in pcm_i16.chunks(frame_size) {
            let input = if self.channels == 1 {
                InterleavedPcm(chunk)
            } else {
                InterleavedPcm(chunk)
            };
            
            let encoded = encoder.encode(input)?;
            mp3_buffer.extend_from_slice(&encoded);
        }
        
        // Flush 剩余数据
        let flushed = encoder.flush::<FlushNoGap>()?;
        mp3_buffer.extend_from_slice(&flushed);
        
        // 写入文件
        output_file.write_all(&mp3_buffer)?;
        
        info!(
            "MP3 编码完成: {} 字节 -> {} 字节",
            pcm_data.len() * 4,
            mp3_buffer.len()
        );
        
        Ok(())
    }
    
    /// 混合两个音频通道（麦克风和扬声器）
    pub fn mix_channels(mic_data: &[f32], speaker_data: &[f32]) -> Vec<f32> {
        let max_len = mic_data.len().max(speaker_data.len());
        let mut mixed = Vec::with_capacity(max_len);
        
        for i in 0..max_len {
            let mic_sample = mic_data.get(i).copied().unwrap_or(0.0);
            let speaker_sample = speaker_data.get(i).copied().unwrap_or(0.0);
            
            // 简单混音：平均两个信号
            let mixed_sample = (mic_sample + speaker_sample) / 2.0;
            
            // 防止削波
            let clamped = mixed_sample.max(-1.0).min(1.0);
            mixed.push(clamped);
        }
        
        debug!("混音完成: {} 样本", mixed.len());
        mixed
    }
    
    /// 重采样音频数据
    pub fn resample(
        input: &[f32],
        source_rate: u32,
        target_rate: u32,
        channels: usize,
    ) -> Result<Vec<f32>> {
        if source_rate == target_rate {
            return Ok(input.to_vec());
        }
        
        use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};
        
        let params = SincInterpolationParameters {
            sinc_len: 256,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 256,
            window: WindowFunction::BlackmanHarris2,
        };
        
        let mut resampler = SincFixedIn::<f32>::new(
            target_rate as f64 / source_rate as f64,
            2.0,
            params,
            input.len() / channels,
            channels,
        )?;
        
        // 将交错数据转换为平面数据
        let mut planes: Vec<Vec<f32>> = vec![Vec::new(); channels];
        for (i, &sample) in input.iter().enumerate() {
            planes[i % channels].push(sample);
        }
        
        // 重采样
        let output_planes = resampler.process(&planes, None)?;
        
        // 将平面数据转换回交错数据
        let mut output = Vec::new();
        let frame_count = output_planes[0].len();
        for i in 0..frame_count {
            for plane in &output_planes {
                output.push(plane[i]);
            }
        }
        
        Ok(output)
    }
}
