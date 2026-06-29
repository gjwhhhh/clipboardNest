use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::task::JoinHandle;
use tokio::time;

use super::{now_secs, SyncDevice};

pub const DISCOVERY_PORT: u16 = 42345;

/// 局域网设备发现服务
pub struct DiscoveryService {
    devices: Arc<Mutex<HashMap<String, SyncDevice>>>,
    local_device: SyncDevice,
    running: Arc<AtomicBool>,
    handles: Mutex<Vec<JoinHandle<()>>>,
}

impl DiscoveryService {
    pub fn new(device_id: String, device_name: String, port: u16) -> Self {
        let address = local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "0.0.0.0".to_string());
        let local_device = SyncDevice {
            id: device_id,
            name: device_name,
            address,
            port,
            last_seen: now_secs(),
        };

        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            local_device,
            running: Arc::new(AtomicBool::new(false)),
            handles: Mutex::new(Vec::new()),
        }
    }

    /// 开始广播和发现设备
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let socket = match UdpSocket::bind(("0.0.0.0", DISCOVERY_PORT)).await {
            Ok(socket) => Arc::new(socket),
            Err(error) => {
                self.running.store(false, Ordering::SeqCst);
                return Err(Box::new(error));
            }
        };
        if let Err(error) = socket.set_broadcast(true) {
            self.running.store(false, Ordering::SeqCst);
            return Err(Box::new(error));
        }

        let broadcast_addr: SocketAddr = match format!("255.255.255.255:{}", DISCOVERY_PORT).parse()
        {
            Ok(address) => address,
            Err(error) => {
                self.running.store(false, Ordering::SeqCst);
                return Err(Box::new(error));
            }
        };
        let mut device_info = self.local_device.clone();

        // 广播线程
        let broadcast_socket = socket.clone();
        let broadcast_running = self.running.clone();
        let broadcast_handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(5));
            while broadcast_running.load(Ordering::SeqCst) {
                interval.tick().await;
                device_info.last_seen = now_secs();
                if let Ok(payload) = serde_json::to_string(&device_info) {
                    let _ = broadcast_socket
                        .send_to(payload.as_bytes(), broadcast_addr)
                        .await;
                }
            }
        });

        // 监听线程
        let devices = self.devices.clone();
        let local_id = self.local_device.id.clone();
        let listen_socket = socket.clone();
        let listen_running = self.running.clone();
        let listen_handle = tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            while listen_running.load(Ordering::SeqCst) {
                if let Ok((len, addr)) = listen_socket.recv_from(&mut buf).await {
                    if let Ok(mut device) = serde_json::from_slice::<SyncDevice>(&buf[..len]) {
                        if device.id != local_id {
                            device.address = addr.ip().to_string();
                            device.last_seen = now_secs();
                            let mut devices = devices.lock().unwrap();
                            devices.insert(device.id.clone(), device);
                        }
                    }
                }
            }
        });

        let mut handles = self.handles.lock().unwrap();
        handles.push(broadcast_handle);
        handles.push(listen_handle);

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        let mut handles = self.handles.lock().unwrap();
        for handle in handles.drain(..) {
            handle.abort();
        }
    }

    /// 获取发现的设备
    pub fn get_devices(&self) -> Vec<SyncDevice> {
        let cutoff = now_secs().saturating_sub(30);
        let devices = self.devices.lock().unwrap();
        devices
            .values()
            .filter(|device| device.last_seen >= cutoff)
            .cloned()
            .collect()
    }

    /// 获取本地设备信息
    pub fn get_local_device(&self) -> SyncDevice {
        self.local_device.clone()
    }
}
