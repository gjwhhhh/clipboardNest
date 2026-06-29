use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tauri::State;
use tokio::task::JoinHandle;

use crate::sync::{client, discovery::DiscoveryService, server, SyncDevice, SyncReport};

pub const SYNC_PORT: u16 = 42346;

pub struct SyncState {
    db_conn: Arc<Mutex<Connection>>,
    discovery: Arc<DiscoveryService>,
    server_handle: Mutex<Option<JoinHandle<()>>>,
}

impl SyncState {
    pub fn new(db_conn: Arc<Mutex<Connection>>, device_id: String, device_name: String) -> Self {
        Self {
            db_conn,
            discovery: Arc::new(DiscoveryService::new(device_id, device_name, SYNC_PORT)),
            server_handle: Mutex::new(None),
        }
    }

    async fn ensure_server_started(&self) -> Result<(), String> {
        if self
            .server_handle
            .lock()
            .map_err(|e| e.to_string())?
            .is_some()
        {
            return Ok(());
        }

        let handle = server::start_server(SYNC_PORT, self.db_conn.clone())
            .await
            .map_err(|e| e.to_string())?;
        let mut server_handle = self.server_handle.lock().map_err(|e| e.to_string())?;
        *server_handle = Some(handle);
        Ok(())
    }

    fn stop_server(&self) -> Result<(), String> {
        if let Some(handle) = self.server_handle.lock().map_err(|e| e.to_string())?.take() {
            handle.abort();
        }
        Ok(())
    }

    fn update_last_sync(&self) -> Result<(), String> {
        let conn = self.db_conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('last_sync', ?1)",
            rusqlite::params![crate::sync::now_secs().saturating_mul(1000).to_string()],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

pub fn get_or_create_device_id(conn: &Connection) -> Result<String, String> {
    if let Some(id) = clipboard_core::storage::database::get_setting(conn, "sync_device_id")
        .map_err(|e| e.to_string())?
    {
        return Ok(id);
    }

    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('sync_device_id', ?1)",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(id)
}

pub fn get_device_name(conn: &Connection) -> Result<String, String> {
    if let Some(name) = clipboard_core::storage::database::get_setting(conn, "sync_device_name")
        .map_err(|e| e.to_string())?
    {
        return Ok(name);
    }

    let name = format!(
        "ClipBoard Mobile {}",
        &uuid::Uuid::new_v4().to_string()[..4]
    );
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('sync_device_name', ?1)",
        rusqlite::params![name],
    )
    .map_err(|e| e.to_string())?;
    Ok(name)
}

#[tauri::command]
pub async fn start_sync(state: State<'_, SyncState>) -> Result<SyncDevice, String> {
    state.ensure_server_started().await?;
    state.discovery.start().await.map_err(|e| e.to_string())?;
    Ok(state.discovery.get_local_device())
}

#[tauri::command]
pub fn stop_sync(state: State<'_, SyncState>) -> Result<(), String> {
    state.discovery.stop();
    state.stop_server()
}

#[tauri::command]
pub fn get_local_sync_device(state: State<'_, SyncState>) -> Result<SyncDevice, String> {
    Ok(state.discovery.get_local_device())
}

#[tauri::command]
pub fn get_discovered_devices(state: State<'_, SyncState>) -> Result<Vec<SyncDevice>, String> {
    Ok(state.discovery.get_devices())
}

#[tauri::command]
pub async fn sync_with_device(
    state: State<'_, SyncState>,
    device_id: String,
) -> Result<SyncReport, String> {
    let device = state
        .discovery
        .get_devices()
        .into_iter()
        .find(|device| device.id == device_id)
        .ok_or("设备未找到")?;

    let report = client::sync_with_device(state.db_conn.clone(), device)
        .await
        .map_err(|e| e.to_string())?;
    state.update_last_sync()?;
    Ok(report)
}

#[tauri::command]
pub async fn sync_all_devices(state: State<'_, SyncState>) -> Result<SyncReport, String> {
    let devices = state.discovery.get_devices();
    let mut total = SyncReport::default();

    for device in devices {
        let report = client::sync_with_device(state.db_conn.clone(), device)
            .await
            .map_err(|e| e.to_string())?;
        total.inserted += report.inserted;
        total.updated += report.updated;
        total.skipped += report.skipped;
    }

    state.update_last_sync()?;
    Ok(total)
}
