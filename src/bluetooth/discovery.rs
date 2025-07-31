//! Bluetooth Discovery Module
//!
//! Handles Bluetooth adapter initialization and scanning nearby devices.
//! Uses the `bluer` crate for async BlueZ interaction.

use bluer::{Adapter, AdapterEvent, Address, Device};
use futures::StreamExt;
use std::{fmt, time::Duration};
/// Bluetooth Adapter Representation
pub struct AdapterController {
    adapter: Adapter,
}

/// Device Identity for each Bluetooth Device scanned
#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub name: String,
    pub address: String,
}
impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.address)
    }
}
impl DeviceInfo {
    /// Builds a `DeviceInfo` from a full `bluer::Device`.
    pub async fn from_device(device: &Device) -> bluer::Result<Self> {
        let name = device.name().await?.unwrap_or_default();
        let address = device.address().to_string();
        Ok(Self { name, address })
    }
}

impl AdapterController {
    ///Initializes & returns a controller that can perform scanning.
    pub async fn initialize() -> bluer::Result<Self> {
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        adapter.set_powered(true).await?;

        Ok(Self { adapter })
    }
    pub async fn get_own_address(&self) -> Result<String, bluer::Error> {
        Ok(self.adapter.address().await?.to_string())
    }
    pub async fn make_discoverable(&self) -> bluer::Result<()> {
        self.adapter.set_discoverable(true).await?;
        self.adapter.set_pairable(true).await?;
        Ok(())
    }
    /// Waits for an incoming Bluetooth connection (used on receiver side)
    /// Polls nearby devices and waits until one becomes connected (receiver side)
    pub async fn wait_for_connection(&self, timeout_secs: u64) -> bluer::Result<DeviceInfo> {
        use tokio::time::{Duration, Instant, sleep};

        println!("[Bluetooth] Waiting for connection (polling mode)...");

        let mut events = self.adapter.discover_devices().await?;
        let deadline = Instant::now() + Duration::from_secs(timeout_secs);

        while Instant::now() < deadline {
            tokio::select! {
                maybe_event = events.next() => {
                    if let Some(AdapterEvent::DeviceAdded(addr)) = maybe_event {
                        let device = self.adapter.device(addr)?;
                        println!("[Bluetooth] New device discovered: {}", addr);

                        // Poll for connection every 250ms for this device
                        let mut retry = 0;
                        while retry < 20 {
                            if device.is_connected().await.unwrap_or(false) {
                                let name = device.name().await?.unwrap_or("Unnamed".into());
                                println!("[Bluetooth] Device {} connected!", name);
                                return Ok(DeviceInfo {
                                    name,
                                    address: addr.to_string()
                                });
                            }
                            sleep(Duration::from_millis(250)).await;
                            retry += 1;
                        }
                    }
                }

                _ = sleep(Duration::from_secs(1)) => {
                    // keep the loop spinning to evaluate timeout
                }
            }
        }

        Err(bluer::Error {
            kind: bluer::ErrorKind::AuthenticationTimeout,
            message: "No device connected in time".to_string(),
        })
    }

    /// Returns a list of discovered device descriptors.
    pub async fn scan_devices(&self, duration: Duration) -> bluer::Result<Vec<DeviceInfo>> {
        let mut discovered = Vec::new();

        let mut events = self.adapter.discover_devices().await?;
        let scan_timeout = tokio::time::sleep(duration);
        tokio::pin!(scan_timeout);
        loop {
            tokio::select! {
                biased;

                _ = &mut scan_timeout => {
                    break;
                }
                maybe_event = events.next() => {
                    if let Some(AdapterEvent::DeviceAdded(addr)) = maybe_event {
                        if let Ok(name) = fetch_device_name(&self.adapter, addr).await {
                            discovered.push(DeviceInfo {
                                name,
                                address: addr.to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(discovered)
    }

    ///Connects to a bluetooth device
    pub async fn connect_to_device(&self, device_info: &DeviceInfo) -> bluer::Result<Device> {
        let addr: Address = device_info.address.parse().map_err(|_| bluer::Error {
            kind: bluer::ErrorKind::InvalidArguments,
            message: "Invalid bluetooth address".to_string(),
        })?;
        let device = loop {
            if let Ok(dev) = self.adapter.device(addr) {
                if dev.is_connected().await.unwrap_or(false) || dev.name().await.is_ok() {
                    break dev;
                }
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        };
        if let Err(e) = device.pair().await {
            if e.to_string().contains("Already Exists") {
                println!("[Connecting] Device already exists, continuing...");
            } else {
                return Err(e);
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        device.connect().await?;
        let mut retry = 0;
        while !device.is_connected().await? && retry < 10 {
            tokio::time::sleep(Duration::from_millis(200)).await;
            retry += 1;
        }
        if device.is_connected().await? {
            Ok(device)
        } else {
            Err(bluer::Error {
                kind: bluer::ErrorKind::Failed,
                message: "Device connection failed".to_string(),
            })
        }
    }

    // pub async fn send_message(&self, device_address: &str, message: &Value) -> Result<(), String> {
    //     let mut socket = BtSocket::new(BtProtocol::RFCOMM)
    //         .map_err(|e| format!("Failed to create socket: {:?}", e))?;

    //     let bt_addr =
    //         BtAddr::from_str(device_address).map_err(|e| format!("Invalid address: {:?}", e))?;

    //     socket
    //         .connect(bt_addr)
    //         .map_err(|e| format!("Connection failed: {:?}", e))?;

    //     let message_str = message.to_string();
    //     socket
    //         .write_all(message_str.as_bytes())
    //         .map_err(|e| format!("Write failed: {}", e))?;

    //     println!("[BT] Message sent to {}: {}", device_address, message_str);
    //     Ok(())
    // }

    // pub async fn receive_bluetooth_message(&self, device_address: &str) -> Result<String, String> {
    //     let mut socket = BtSocket::new(BtProtocol::RFCOMM)
    //         .map_err(|e| format!("Failed to create socket: {:?}", e))?;

    //     let bt_addr = BtAddr::from_str(device_address)
    //         .map_err(|_| "Invalid Bluetooth address format".to_string())?;

    //     // Connect to listen for messages
    //     socket
    //         .connect(bt_addr)
    //         .map_err(|e| format!("Connection failed: {:?}", e))?;

    //     // Read incoming message
    //     let mut buffer = [0u8; 1024]; // 1KB buffer should be enough for JSON
    //     let bytes_read = socket
    //         .read(&mut buffer)
    //         .map_err(|e| format!("Failed to read message: {}", e))?;

    //     // Convert bytes to string
    //     let message = String::from_utf8(buffer[..bytes_read].to_vec())
    //         .map_err(|e| format!("Invalid UTF-8 message: {}", e))?;

    //     println!(
    //         "[BT] Received message from {}: {}",
    //         device_address,
    //         message.trim()
    //     );
    //     Ok(message.trim().to_string()) // Remove any trailing newlines
    // }
}

/// Fetches the user-friendly name of a device given its address.
///
/// If the name is unavailable, returns "NULL".
async fn fetch_device_name(adapter: &Adapter, addr: Address) -> bluer::Result<String> {
    let device = adapter.device(addr)?;
    let name = device.name().await?;
    Ok(name.unwrap_or_else(|| "NULL".to_string()))
}
