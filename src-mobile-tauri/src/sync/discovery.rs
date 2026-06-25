use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time;

use super::SyncDevice;

/// 局域网设备发现服务
pub struct DiscoveryService {
    devices: Arc<Mutex<HashMap<String, SyncDevice>>>,
    local_device: SyncDevice,
}

impl DiscoveryService {
    pub fn new(device_name: String, port: u16) -> Self {
        let local_device = SyncDevice {
            id: uuid::Uuid::new_v4().to_string(),
            name: device_name,
            address: "0.0.0.0".to_string(),
            port,
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            local_device,
        }
    }

    /// 开始广播和发现设备
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        socket.set_broadcast(true)?;

        let broadcast_addr = "255.255.255.255:12345";
        let device_info = serde_json::to_string(&self.local_device)?;

        // 广播线程
        let broadcast_socket = socket.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                let _ = broadcast_socket
                    .send_to(device_info.as_bytes(), broadcast_addr)
                    .await;
            }
        });

        // 监听线程
        let devices = self.devices.clone();
        let local_id = self.local_device.id.clone();
        let listen_socket = socket.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                if let Ok((len, _addr)) = listen_socket.recv_from(&mut buf).await {
                    if let Ok(mut device) = serde_json::from_slice::<SyncDevice>(&buf[..len]) {
                        if device.id != local_id {
                            device.last_seen = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            let mut devices = devices.lock().unwrap();
                            devices.insert(device.id.clone(), device);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// 获取发现的设备
    pub fn get_devices(&self) -> Vec<SyncDevice> {
        let devices = self.devices.lock().unwrap();
        devices.values().cloned().collect()
    }

    /// 获取本地设备信息
    pub fn get_local_device(&self) -> SyncDevice {
        self.local_device.clone()
    }
}
