// 智能麦克风占用检测模块
// 通过Windows Core Audio API检测哪些进程正在使用麦克风

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::time::Duration;

#[cfg(target_os = "windows")]
use windows::{
    core::*,
    Win32::Media::Audio::*,
    Win32::System::Com::*,
    Win32::Foundation::*,
    Win32::System::Threading::*,
};

#[derive(Debug, Clone)]
pub struct AudioSession {
    pub process_id: u32,
    pub process_name: String,
    pub is_capture: bool,  // true = 麦克风, false = 播放
    pub session_id: String,
}

pub struct MicrophoneDetector {
    blacklist: HashSet<String>,
    last_active_sessions: HashSet<u32>,
}

impl MicrophoneDetector {
    pub fn new() -> Self {
        Self {
            blacklist: HashSet::new(),
            last_active_sessions: HashSet::new(),
        }
    }

    pub fn add_to_blacklist(&mut self, process_name: String) {
        self.blacklist.insert(process_name.to_lowercase());
    }

    pub fn remove_from_blacklist(&mut self, process_name: &str) {
        self.blacklist.remove(&process_name.to_lowercase());
    }

    pub fn set_blacklist(&mut self, processes: Vec<String>) {
        self.blacklist = processes.into_iter()
            .map(|s| s.to_lowercase())
            .collect();
    }

    #[cfg(target_os = "windows")]
    pub fn detect_active_sessions(&mut self) -> Result<Vec<AudioSession>> {
        unsafe {
            // 初始化COM
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            // 创建设备枚举器
            let enumerator: IMMDeviceEnumerator = CoCreateInstance(
                &MMDeviceEnumerator,
                None,
                CLSCTX_ALL,
            )?;

            let mut all_sessions = Vec::new();

            // 检测麦克风设备的会话
            if let Ok(capture_sessions) = self.get_device_sessions(&enumerator, eCapture) {
                all_sessions.extend(capture_sessions);
            }

            // 过滤掉黑名单中的应用
            let active_sessions: Vec<AudioSession> = all_sessions
                .into_iter()
                .filter(|session| {
                    !self.blacklist.contains(&session.process_name.to_lowercase())
                })
                .collect();

            // 记录当前活跃的会话
            self.last_active_sessions = active_sessions
                .iter()
                .map(|s| s.process_id)
                .collect();

            CoUninitialize();
            Ok(active_sessions)
        }
    }

    #[cfg(target_os = "windows")]
    unsafe fn get_device_sessions(
        &self,
        enumerator: &IMMDeviceEnumerator,
        data_flow: EDataFlow,
    ) -> Result<Vec<AudioSession>> {
        let mut sessions = Vec::new();

        // 获取默认设备
        let device = enumerator.GetDefaultAudioEndpoint(
            data_flow,
            eConsole,
        )?;

        // 激活会话管理器
        let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;

        // 获取会话枚举器
        let session_enumerator = session_manager.GetSessionEnumerator()?;
        let count = session_enumerator.GetCount()?;

        for i in 0..count {
            if let Ok(session_control) = session_enumerator.GetSession(i) {
                if let Ok(session2) = session_control.cast::<IAudioSessionControl2>() {
                    // 获取进程ID
                    if let Ok(process_id) = session2.GetProcessId() {
                        if process_id == 0 {
                            continue; // 跳过系统会话
                        }

                        // 获取进程名称
                        let process_name = self.get_process_name(process_id)
                            .unwrap_or_else(|_| format!("Process_{}", process_id));

                        // 获取会话ID
                        let session_id = session2.GetSessionInstanceIdentifier()
                            .map(|s| s.to_string().unwrap_or_default())
                            .unwrap_or_default();

                        // 检查会话状态
                        if let Ok(state) = session_control.GetState() {
                            if state == AudioSessionStateActive {
                                sessions.push(AudioSession {
                                    process_id,
                                    process_name,
                                    is_capture: data_flow == eCapture,
                                    session_id,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    #[cfg(target_os = "windows")]
    unsafe fn get_process_name(&self, process_id: u32) -> Result<String> {
        let process_handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            process_id,
        )?;

        let mut buffer = vec![0u16; 1024];
        let mut size = buffer.len() as u32;

        if QueryFullProcessImageNameW(
            process_handle,
            PROCESS_NAME_WIN32,
            &mut buffer,
            &mut size,
        ).is_ok() {
            let path = String::from_utf16_lossy(&buffer[..size as usize]);
            // 提取文件名
            if let Some(filename) = path.split('\\').last() {
                return Ok(filename.to_string());
            }
        }

        let _ = CloseHandle(process_handle);
        Err(anyhow::anyhow!("Failed to get process name"))
    }

    /// 检查是否有新的麦克风使用者(不在黑名单中)
    pub fn has_new_microphone_user(&self, sessions: &[AudioSession]) -> bool {
        sessions.iter().any(|s| s.is_capture && s.process_id != 0)
    }

    /// 获取正在使用麦克风的应用名称列表
    pub fn get_active_apps(&self, sessions: &[AudioSession]) -> Vec<String> {
        sessions.iter()
            .filter(|s| s.is_capture)
            .map(|s| s.process_name.clone())
            .collect()
    }
}

#[cfg(not(target_os = "windows"))]
impl MicrophoneDetector {
    pub fn detect_active_sessions(&mut self) -> Result<Vec<AudioSession>> {
        Err(anyhow::anyhow!("Only supported on Windows"))
    }

    pub fn has_new_microphone_user(&self, _sessions: &[AudioSession]) -> bool {
        false
    }

    pub fn get_active_apps(&self, _sessions: &[AudioSession]) -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blacklist() {
        let mut detector = MicrophoneDetector::new();
        detector.add_to_blacklist("chrome.exe".to_string());
        detector.add_to_blacklist("firefox.exe".to_string());
        
        assert!(detector.blacklist.contains("chrome.exe"));
        assert!(detector.blacklist.contains("firefox.exe"));
        
        detector.remove_from_blacklist("chrome.exe");
        assert!(!detector.blacklist.contains("chrome.exe"));
    }
}
