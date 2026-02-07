use anyhow::{Context, Result};
use mp3lame::{Lame, BitRate, Quality};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct Mp3Encoder {
    lame: Lame,
    output_file: File,
    sample_rate: u32,
}

impl Mp3Encoder {
    pub fn new(
        output_path: &Path,
        sample_rate: u32,
        bit_rate: u32,
        quality: u8,
    ) -> Result<Self> {
        let mut lame = Lame::new().context("Failed to initialize LAME")?;

        // 设置参数
        lame.set_sample_rate(sample_rate)?;
        lame.set_num_channels(1)?; // 单声道
        lame.set_kilobitrate(bit_rate as i32)?;
        
        // 设置质量 (0-9)
        let q = match quality {
            0 => Quality::Best,
            1 => Quality::Best,
            2 => Quality::Good,
            3..=4 => Quality::Good,
            5..=6 => Quality::Medium,
            7..=8 => Quality::Bad,
            _ => Quality::Worst,
        };
        lame.set_quality(q)?;

        lame.init_params()?;

        let output_file = File::create(output_path)
            .with_context(|| format!("Failed to create output file: {:?}", output_path))?;

        Ok(Self {
            lame,
            output_file,
            sample_rate,
        })
    }

    pub fn encode_samples(&mut self, samples: &[f32]) -> Result<()> {
        if samples.is_empty() {
            return Ok(());
        }

        // 转换 f32 样本到 i16
        let pcm: Vec<i16> = samples
            .iter()
            .map(|&s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
            .collect();

        // MP3 编码缓冲区
        let mut mp3_buffer = vec![0u8; pcm.len() * 5 / 4 + 7200];

        // 编码
        let encoded_size = self.lame.encode(&pcm, &mut mp3_buffer)?;

        if encoded_size > 0 {
            self.output_file.write_all(&mp3_buffer[..encoded_size])?;
        }

        Ok(())
    }

    pub fn finish(mut self) -> Result<()> {
        // 刷新编码器
        let mut mp3_buffer = vec![0u8; 7200];
        let flush_size = self.lame.flush(&mut mp3_buffer)?;

        if flush_size > 0 {
            self.output_file.write_all(&mp3_buffer[..flush_size])?;
        }

        self.output_file.flush()?;
        
        tracing::info!("MP3 encoding finished");
        Ok(())
    }
}
