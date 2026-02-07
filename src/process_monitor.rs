use sysinfo::{ProcessExt, System, SystemExt};
use std::collections::HashSet;
use std::time::Duration;
use tokio::time;
use tracing::{debug, info};

pub struct ProcessMonitor {
    system: System,
    monitored_apps: HashSet<String>,
    active_apps: HashSet<String>,
}

impl ProcessMonitor {
    pub fn new(monitored_apps: Vec<String>) -> Self {
        let mut set = HashSet::new();
        for app in monitored_apps {
            set.insert(app.to_lowercase());
        }
        
        Self {
            system: System::new_all(),
            monitored_apps: set,
            active_apps: HashSet::new(),
        }
    }
    
    /// 刷新进程列表
    pub fn refresh(&mut self) {
        self.system.refresh_processes();
    }
    
    /// 检查监控的应用是否正在运行
    pub fn check_apps_running(&mut self) -> bool {
        self.refresh();
        
        let mut current_active = HashSet::new();
        
        for (_pid, process) in self.system.processes() {
            let process_name = process.name().to_lowercase();
            
            for monitored in &self.monitored_apps {
                if process_name.contains(monitored) || 
                   process_name.contains(&monitored.trim_end_matches(".exe")) {
                    current_active.insert(monitored.clone());
                }
            }
        }
        
        // 检测新启动的应用
        for app in &current_active {
            if !self.active_apps.contains(app) {
                info!("检测到应用启动: {}", app);
            }
        }
        
        // 检测关闭的应用
        for app in &self.active_apps {
            if !current_active.contains(app) {
                info!("检测到应用关闭: {}", app);
            }
        }
        
        self.active_apps = current_active;
        !self.active_apps.is_empty()
    }
    
    /// 获取当前活动的应用列表
    pub fn get_active_apps(&self) -> Vec<String> {
        self.active_apps.iter().cloned().collect()
    }
}

pub async fn monitor_processes(
    monitored_apps: Vec<String>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> tokio::sync::watch::Receiver<bool> {
    let (tx, rx) = tokio::sync::watch::channel(false);
    let mut monitor = ProcessMonitor::new(monitored_apps);
    
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(2));
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let is_running = monitor.check_apps_running();
                    let _ = tx.send(is_running);
                }
                _ = shutdown.changed() => {
                    debug!("进程监控已停止");
                    break;
                }
            }
        }
    });
    
    rx
}
