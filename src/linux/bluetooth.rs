use bluer::{Adapter, AdapterEvent, Address, Session};
use futures::StreamExt;
use std::time::Duration;
use std::{fmt};

use std::sync::{Arc, Mutex};
use uuid::Uuid;
use bluer::gatt::local::{Application, Characteristic, CharacteristicRead, CharacteristicReadRequest, ReqError, Service};

pub struct AdapterController {
    adapter: Adapter,
    session: Session
}

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

impl AdapterController {
    ///Initializes & returns a controller that can perform scanning.
    pub async fn initialize() -> bluer::Result<Self> {
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        adapter.set_powered(true).await?;

        Ok(Self { adapter, session })
    }
    pub async fn get_own_address(&self) -> Result<String, bluer::Error> {
        Ok(self.adapter.address().await?.to_string())
    }
    pub async fn make_discoverable(&self) -> bluer::Result<()> {
        self.adapter.set_discoverable(true).await?;
        self.adapter.set_pairable(true).await?;
        Ok(())
    }
    pub async fn scan_devices(&self, timeout_secs: Duration) -> Result<Vec<DeviceInfo>, Box<dyn std::error::Error>> {
        use tokio::time::{Duration, Instant, sleep};

        println!("[Bluetooth] Scanning for nearby devices...");

        let mut events = self.adapter.discover_devices().await?;
        let deadline = Instant::now() + timeout_secs;

        let mut found_devices = vec![];

        while Instant::now() < deadline {
            tokio::select! {
                maybe_event = events.next() => {
                    if let Some(AdapterEvent::DeviceAdded(addr)) = maybe_event {
                        let device = self.adapter.device(addr)?;
                        let name = device.name().await?.unwrap_or("Unnamed".into());

                        let info = DeviceInfo {
                            name: name.clone(),
                            address: addr.to_string(),
                        };

                        println!("  → Found: {}", info);
                        found_devices.push(info);
                    }
                }

                _ = sleep(Duration::from_millis(200)) => {
                    // just keep spinning for timeout
                }
            }
        }

        println!("[Bluetooth] Scan complete. Found {} devices.", found_devices.len());
        Ok(found_devices)
    }

    pub async fn connect_to_device(&self, address: &str) -> bluer::Result<()> {
        let addr: Address = address.parse().map_err(|e| bluer::Error {
            kind: bluer::ErrorKind::Failed,
            message: format!("Invalid Bluetooth address: {}", e),
        })?;
    
        let device = self.adapter.device(addr)?;
        println!("[Bluetooth] Connecting to device {}...", addr);
        device.connect().await?;
        println!("[Bluetooth] Connected to device {}", addr);
    
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

    pub async fn serve_gatt(&self, key_data: Vec<u8>, expected_mac: bluer::Address) -> Result<(), Box<dyn std::error::Error>> {
        let key_data = Arc::new(Mutex::new(key_data));
        let expected_mac = Arc::new(expected_mac);
        
        let app = Application {
            services: vec![Service {
                uuid: Uuid::parse_str("12345678-1234-5678-1234-56789abcdef0")?,
                primary: true,
                characteristics: vec![Characteristic {
                    uuid: Uuid::parse_str("abcdef12-3456-7890-abcd-ef1234567890")?,
                    read: Some(CharacteristicRead {
                        read: true,
                        fun: Box::new({
                            let key_data = Arc::clone(&key_data);
                            let expected_mac = Arc::clone(&expected_mac);
                            move |req: CharacteristicReadRequest| {
                                let key_data = Arc::clone(&key_data);
                                let expected_mac = Arc::clone(&expected_mac);
                                Box::pin(async move {
                                    // Check if the requesting device's MAC matches our expected macOS device
                                    if req.device_address == *expected_mac {
                                        let data = key_data.lock().unwrap().clone();
                                        println!("[Bluetooth] Key read by authorized macOS device: {:?}", req.device_address);
                                        Ok(data)
                                    } else {
                                        println!("[Bluetooth] Unauthorized read attempt from: {:?}, expected: {:?}", req.device_address, *expected_mac);
                                        Err(ReqError::NotPermitted)
                                    }
                                })
                            }
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        
        println!("[Bluetooth] Serving GATT with fling key for device: {:?}...", expected_mac);
        self.adapter.serve_gatt_application(app).await?;
        Ok(())
    }
}

