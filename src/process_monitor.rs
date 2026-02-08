#[cfg(target_os = "windows")]
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Threading::*,
    Win32::System::Diagnostics::ToolHelp::*,
};

#[cfg(target_os = "windows")]
use std::collections::HashSet;

#[cfg(target_os = "windows")]
pub struct ProcessMonitor {
    monitored_apps: HashSet<String>,
}

#[cfg(target_os = "windows")]
impl ProcessMonitor {
    pub fn new(apps: Vec<String>) -> Self {
        let monitored_apps = apps.into_iter()
            .map(|s| s.to_lowercase())
            .collect();
        
        Self { monitored_apps }
    }

    pub fn is_call_active(&self) -> bool {
        match self.get_running_processes() {
            Ok(processes) => {
                for process in processes {
                    if self.monitored_apps.contains(&process.to_lowercase()) {
                        tracing::debug!("Detected running app: {}", process);
                        return true;
                    }
                }
                false
            }
            Err(e) => {
                tracing::error!("Failed to get running processes: {}", e);
                false
            }
        }
    }

    fn get_running_processes(&self) -> anyhow::Result<Vec<String>> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
            
            let mut processes = Vec::new();
            let mut entry = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };

            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    let exe_name = String::from_utf16_lossy(
                        &entry.szExeFile
                            .iter()
                            .take_while(|&&c| c != 0)
                            .copied()
                            .collect::<Vec<u16>>(),
                    );
                    
                    if !exe_name.is_empty() {
                        processes.push(exe_name);
                    }

                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }

            let _ = CloseHandle(snapshot);
            Ok(processes)
        }
    }
}

// 非 Windows 平台的桩实现
#[cfg(not(target_os = "windows"))]
pub struct ProcessMonitor {
    monitored_apps: Vec<String>,
}

#[cfg(not(target_os = "windows"))]
impl ProcessMonitor {
    pub fn new(apps: Vec<String>) -> Self {
        tracing::warn!("Process monitoring is only supported on Windows");
        Self { monitored_apps: apps }
    }

    pub fn is_call_active(&self) -> bool {
        // 非 Windows 平台总是返回 false
        false
    }
}
