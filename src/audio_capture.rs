use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Sample, SampleFormat, SampleRate, StreamConfig};
use std::sync::{Arc, Mutex};
use anyhow::{anyhow, Result};
use tracing::{info, warn, error};

pub struct AudioCapture {
    host: cpal::Host,
}

impl AudioCapture {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        Ok(Self { host })
    }
    
    /// 获取默认输入设备（麦克风）
    pub fn get_input_device(&self) -> Result<Device> {
        self.host
            .default_input_device()
            .ok_or_else(|| anyhow!("未找到默认输入设备"))
    }
    
    /// 获取默认输出设备（扬声器）
    pub fn get_output_device(&self) -> Result<Device> {
        self.host
            .default_output_device()
            .ok_or_else(|| anyhow!("未找到默认输出设备"))
    }
    
    /// 列出所有可用设备
    pub fn list_devices(&self) -> Result<Vec<String>> {
        let mut devices = Vec::new();
        
        if let Ok(input_devices) = self.host.input_devices() {
            for device in input_devices {
                if let Ok(name) = device.name() {
                    devices.push(format!("输入: {}", name));
                }
            }
        }
        
        if let Ok(output_devices) = self.host.output_devices() {
            for device in output_devices {
                if let Ok(name) = device.name() {
                    devices.push(format!("输出: {}", name));
                }
            }
        }
        
        Ok(devices)
    }
    
    /// 获取设备的默认配置
    pub fn get_device_config(&self, device: &Device) -> Result<StreamConfig> {
        let default_config = device.default_input_config()?;
        Ok(default_config.into())
    }
    
    /// 创建音频捕获流
    pub fn create_capture_stream(
        &self,
        device: &Device,
        config: &StreamConfig,
        sample_rate: u32,
        buffer: Arc<Mutex<Vec<f32>>>,
    ) -> Result<cpal::Stream> {
        let channels = config.channels as usize;
        let target_sample_rate = sample_rate;
        let source_sample_rate = config.sample_rate.0;
        
        info!(
            "创建捕获流 - 源采样率: {}, 目标采样率: {}, 声道数: {}",
            source_sample_rate, target_sample_rate, channels
        );
        
        let buffer_clone = Arc::clone(&buffer);
        
        let stream = match config.sample_format {
            SampleFormat::F32 => {
                device.build_input_stream(
                    config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer_clone.lock().unwrap();
                        buf.extend_from_slice(data);
                    },
                    |err| error!("音频流错误: {}", err),
                    None,
                )?
            }
            SampleFormat::I16 => {
                device.build_input_stream(
                    config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer_clone.lock().unwrap();
                        for &sample in data {
                            buf.push(sample.to_float_sample());
                        }
                    },
                    |err| error!("音频流错误: {}", err),
                    None,
                )?
            }
            SampleFormat::U16 => {
                device.build_input_stream(
                    config,
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer_clone.lock().unwrap();
                        for &sample in data {
                            buf.push(sample.to_float_sample());
                        }
                    },
                    |err| error!("音频流错误: {}", err),
                    None,
                )?
            }
            _ => {
                return Err(anyhow!("不支持的采样格式"));
            }
        };
        
        Ok(stream)
    }
}

/// Windows 环境下捕获扬声器输出（使用 WASAPI Loopback）
#[cfg(target_os = "windows")]
pub mod windows_loopback {
    use super::*;
    
    pub fn get_loopback_device() -> Result<Device> {
        let host = cpal::default_host();
        
        // 尝试获取默认输出设备
        let output_device = host
            .default_output_device()
            .ok_or_else(|| anyhow!("未找到默认输出设备"))?;
        
        Ok(output_device)
    }
    
    /// 创建 loopback 捕获流
    pub fn create_loopback_stream(
        device: &Device,
        buffer: Arc<Mutex<Vec<f32>>>,
    ) -> Result<cpal::Stream> {
        let config = device.default_output_config()?;
        let config: StreamConfig = config.into();
        
        info!("创建 Loopback 流 - 采样率: {}, 声道数: {}", 
              config.sample_rate.0, config.channels);
        
        let buffer_clone = Arc::clone(&buffer);
        
        let stream = match config.sample_format {
            SampleFormat::F32 => {
                device.build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer_clone.lock().unwrap();
                        buf.extend_from_slice(data);
                    },
                    |err| error!("Loopback 流错误: {}", err),
                    None,
                )?
            }
            SampleFormat::I16 => {
                device.build_input_stream(
                    &config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer_clone.lock().unwrap();
                        for &sample in data {
                            buf.push(sample.to_float_sample());
                        }
                    },
                    |err| error!("Loopback 流错误: {}", err),
                    None,
                )?
            }
            _ => {
                return Err(anyhow!("不支持的采样格式"));
            }
        };
        
        Ok(stream)
    }
}
