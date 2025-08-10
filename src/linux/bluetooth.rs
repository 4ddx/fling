use bluer::{Adapter, AdapterEvent, Address, Session};
use futures::{StreamExt};
use std::fmt;
use std::process::Command;
use bluer::Uuid;
use std::error::Error;
use std::time::{Duration, Instant};
use tokio::time::sleep;
pub struct AdapterController {
    adapter: Adapter
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
    pub async fn initialize() -> bluer::Result<Self> {
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        adapter.set_powered(true).await?;

        Ok(Self { adapter })
    }
    pub async fn scan_devices(
        &self,
        timeout_secs: Duration,
    ) -> Result<Vec<DeviceInfo>, Box<dyn std::error::Error>> {
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
                }
            }
        }

        println!(
            "[Bluetooth] Scan complete. Found {} devices.",
            found_devices.len()
        );
        Ok(found_devices)
    }

    pub async fn serve_gatt(
        &self,
        key_data: Vec<u8>,
        expected_mac: bluer::Address,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use tokio::sync::Mutex;
        use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
        use std::time::Duration;
        use bluer::gatt::local::{
            Application, Service, Characteristic, CharacteristicRead,
            CharacteristicReadRequest
        };
        use bluer::adv::{Advertisement, Type};
        use uuid::Uuid;
    
        let key_data = Arc::new(Mutex::new(key_data));
        let expected_mac = Arc::new(expected_mac);
        let key_read = Arc::new(AtomicBool::new(false));
    
        let service_uuid = Uuid::parse_str("12345678-1234-5678-1234-56789abcdef0")?;
        let char_uuid = Uuid::parse_str("abcdef12-3456-7890-abcd-ef1234567890")?;
    
        let app = Application {
            services: vec![Service {
                uuid: service_uuid,
                primary: true,
                characteristics: vec![Characteristic {
                    uuid: char_uuid,
                    read: Some(CharacteristicRead {
                        read: true,
                        fun: {
                            let key_data = Arc::clone(&key_data);
                            let expected_mac = Arc::clone(&expected_mac);
                            let key_read = Arc::clone(&key_read);
                            Box::new(move |req: CharacteristicReadRequest| {
                                let key_data = Arc::clone(&key_data);
                                let expected_mac = Arc::clone(&expected_mac);
                                let key_read = Arc::clone(&key_read);
                                Box::pin(async move {
                                    // For testing, you might want to temporarily remove MAC check
                                    println!(
                                        "[Bluetooth] Read request from: {:?}, expected: {:?}",
                                        req.device_address, *expected_mac
                                    );
                                    
                                    // Temporarily allow any device for testing
                                    // if req.device_address == *expected_mac {
                                        let data = key_data.lock().await.clone();
                                        println!(
                                            "[Bluetooth] Key read by device: {:?}",
                                            req.device_address
                                        );
                                        key_read.store(true, Ordering::SeqCst);
                                        Ok(data)
                                    // } else {
                                    //     println!(
                                    //         "[Bluetooth] Unauthorized read attempt from: {:?}",
                                    //         req.device_address
                                    //     );
                                    //     Err(ReqError::NotPermitted)
                                    // }
                                })
                            })
                        },
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
    
        println!("[Bluetooth] Registering GATT application...");
        
        let app_handle = self.adapter.serve_gatt_application(app).await?;
        
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        let adv = Advertisement {
            advertisement_type: Type::Peripheral,
            service_uuids: vec![service_uuid].into_iter().collect(),
            discoverable: Some(true),
            local_name: Some("fling-sender".to_string()),
            ..Default::default()
        };
    
        println!("[Bluetooth] Starting advertisement with service UUID: {}", service_uuid);
        let adv_handle = self.adapter.advertise(adv).await?;
    
        println!("[Bluetooth] GATT server ready, waiting for device to read key (30s timeout)...");
        
        for i in 0..30 {
            if key_read.load(Ordering::SeqCst) {
                println!("[Bluetooth] Key successfully read, waiting 4s before terminating...");
                tokio::time::sleep(Duration::from_secs(4)).await;
                break;
            }
            if i % 5 == 0 {
                println!("[Bluetooth] Still waiting... ({}/30s)", i);
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    
        if !key_read.load(Ordering::SeqCst) {
            println!("[Bluetooth] Timeout: Key was not read within 30 seconds");
        }
    
        // Clean up
        drop(adv_handle);
        drop(app_handle);
        
        println!("[Bluetooth] GATT server terminated");
        Ok(())
    }
}

pub async fn receive_fling_key() -> Result<Vec<u8>, Box<dyn Error>> {

    let service_uuid = Uuid::parse_str("12345678-1234-5678-1234-56789abcdef0")?;
    let char_uuid = Uuid::parse_str("abcdef12-3456-7890-abcd-ef1234567890")?;

    let session: Session = Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;
    println!("[Linux] Adapter powered: {}", adapter.name());

    println!("[Linux] Scanning for fling sender ({}s timeout)...", 30);
    let mut events = adapter.discover_devices().await?;

    let scan_deadline = Instant::now() + Duration::from_secs(30);
    let mut found_addr: Option<Address> = None;

    while Instant::now() < scan_deadline {
        let maybe_evt = tokio::time::timeout(Duration::from_secs(2), events.next()).await;
        match maybe_evt {
            Ok(Some(evt)) => {
                if let AdapterEvent::DeviceAdded(addr) = evt {
                    let device = adapter.device(addr)?;
                    let name: Option<String> = device.name().await?;
                    println!("[Scan] Found device: {:?} ({})", name, addr);

                    if let Ok(opt_uuids) = device.uuids().await {
                        if let Some(uuids) = opt_uuids {
                            if uuids.contains(&service_uuid) {
                                println!("[Scan] Device {} advertises fling service", addr);
                                found_addr = Some(addr);
                                break;
                            }
                        }
                    }
                } 
            }
            Ok(None) => {
                break;
            }
            Err(_) => {
            }
        }
    }

    let addr = match found_addr {
        Some(a) => a,
        None => {
            return Err("No fling device found during scan".into());
        }
    };

    drop(events);
    let device = adapter.device(addr)?;
    println!("[Linux] Connecting to device {}", addr);
    device.connect().await?;

    // Wait for device to be connected (small polling loop)
    let conn_deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let connected = device.is_connected().await?;
        if connected {
            break;
        }
        if Instant::now() >= conn_deadline {
            return Err("Timed out waiting for device to connect".into());
        }
        sleep(Duration::from_millis(200)).await;
    }
    println!("[Linux] Device connected.");
    let gatt_services: Vec<bluer::gatt::remote::Service> = device.services().await?;
    println!("[Linux] Discovered {} services", gatt_services.len());

    // -------- DEMO LOG: list all services & characteristics (remove later) --------
    for svc in &gatt_services {
        let svc_uuid = svc.uuid().await?;
        println!("  Service UUID: {}", svc_uuid);
        let chars = svc.characteristics().await?;
        println!("    {} characteristics", chars.len());
        for ch in &chars {
            let ch_uuid = ch.uuid().await?;
            println!("      Characteristic UUID: {}", ch_uuid);
        }
    }
    // ---------------------------------------------------------------------------

    let mut fling_service_opt: Option<bluer::gatt::remote::Service> = None;
    for svc in gatt_services {
        let svc_uuid = svc.uuid().await?;
        if svc_uuid == service_uuid {
            fling_service_opt = Some(svc);
            break;
        }
    }

    let fling_service = match fling_service_opt {
        Some(s) => s,
        None => {
            let _ = device.disconnect().await;
            return Err("Fling service not found on device".into());
        }
    };

    let chars = fling_service.characteristics().await?;
    let mut target_char_opt: Option<bluer::gatt::remote::Characteristic> = None;
    for ch in chars {
        let ch_uuid = ch.uuid().await?;
        if ch_uuid == char_uuid {
            target_char_opt = Some(ch);
            break;
        }
    }

    let target_char = match target_char_opt {
        Some(c) => c,
        None => {
            let _ = device.disconnect().await;
            return Err("Fling characteristic not found in service".into());
        }
    };

    println!("[Linux] Reading fling characteristic...");
    let key: Vec<u8> = target_char.read().await?;

    println!("[Linux] Read {} bytes from fling characteristic", key.len());

    device.disconnect().await?;
    println!("[Linux] Disconnected.");

    Ok(key)
}

pub fn get_bluetooth_mac() -> Option<String> {
    let output = Command::new("bluetoothctl")
        .arg("list")
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(mac) = stdout.split_whitespace().nth(1) {
        Some(mac.to_string())
    } else {
        None
    }
}