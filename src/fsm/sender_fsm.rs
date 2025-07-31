use std::time::Duration;
use dialoguer::{theme::ColorfulTheme, Select};

use crate::bluetooth::{self, discovery::DeviceInfo};
use crate::tunnel;

#[derive(Debug)]
pub enum SenderState {
    Scanning,
    Connecting(DeviceInfo),
    StartingHotspot(DeviceInfo),   // Replaces SettingUpNetwork
    WaitingForJoin(DeviceInfo),    // NEW: Wait for Wi-Fi client
    Sending,
    SendSuccess,
    SendFailed,
    NoDevicesFound,
    ConnectionFailed,
}

pub async fn start_sender_fsm(filepath: &str) -> SenderState {
    use SenderState::*;
    println!("[Scanning] Initializing Bluetooth...");

    // Initialize AdapterController
    let adapter = match bluetooth::discovery::AdapterController::initialize().await {
        Ok(controller) => controller,
        Err(e) => {
            eprintln!("[Scanning] Failed to initialize Bluetooth: {}", e);
            return NoDevicesFound;
        }
    };
    let mut state = Scanning;
    loop {
        state = match state {
            Scanning => {
                println!("[Scanning] Searching for nearby receivers...");
                let devices = match adapter.scan_devices(Duration::from_secs(10)).await {
                    Ok(devices) => devices,
                    Err(e) => {
                        eprintln!("[Scanning] Scan failed: {}", e);
                        return NoDevicesFound;
                    }
                };

                if devices.is_empty() {
                    println!("[Scanning] No receivers found.");
                    return NoDevicesFound;
                }

                println!("[Scanning] Device(s) found! Selecting device...");
                let index = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select a device to connect to")
                    .items(&devices)
                    .default(0)
                    .interact()
                    .unwrap_or(0);

                let chosen = devices[index].clone();
                println!("[Scanning] Selected device: {}", chosen);

                Connecting(chosen)
            }

            Connecting(device_info) => {
                println!("[Connecting] Attempting to connect to {}", device_info.name);
                match adapter.connect_to_device(&device_info).await {
                    Ok(_) => {
                        println!("[Connecting] Connected to {}", device_info.name);
                        StartingHotspot(device_info)
                    }
                    Err(e) => {
                        eprintln!("[Connecting] Failed: {}", e);
                        ConnectionFailed
                    }
                }
            }

            StartingHotspot(device_info) => {
                // Use receiver's hostname + BT MAC to create deterministic SSID
                let hostname = device_info.name.clone();
                let mac_fragment = device_info.address.replace(":", "").to_lowercase();
                let suffix = &mac_fragment[mac_fragment.len()-4..];
                let ssid = format!("fling-{}-{}", hostname, suffix);
                let password = format!("fling-{}!", suffix);

                println!("[Hotspot] Creating AP SSID: {} | PW: {}", ssid, password);
                match tunnel::connection::create_wifi_direct_network(&ssid, &password).await {
                    Ok(_) => {
                        println!("[Hotspot] AP live. Waiting for receiver to join...");
                        WaitingForJoin(device_info)
                    }
                    Err(e) => {
                        eprintln!("[Hotspot] Failed: {}", e);
                        ConnectionFailed
                    }
                }
            }

            WaitingForJoin(_device_info) => {
                println!("[WaitingForJoin] Polling for client...");
                match tunnel::connection::wait_for_receiver().await {
                    Ok(_) => {
                        println!("[WaitingForJoin] Receiver joined the network!");
                        Sending
                    }
                    Err(e) => {
                        eprintln!("[WaitingForJoin] Timeout or failure: {}", e);
                        ConnectionFailed
                    }
                }
            }

            Sending => {
                println!("[Sending] Starting file transfer: {}", filepath);
                match tunnel::transfer::send_file(filepath).await {
                    Ok(_) => SendSuccess,
                    Err(e) => {
                        eprintln!("[Sending] Failed: {}", e);
                        SendFailed
                    }
                }
            }            
            
            SendSuccess => {
                println!("[✅] Transfer complete!");
                break SendSuccess;
            }

            SendFailed => {
                println!("[❌] Transfer failed.");
                break SendFailed;
            }

            NoDevicesFound => {
                println!("[NoDevicesFound] Exiting.");
                break NoDevicesFound;
            }

            ConnectionFailed => {
                println!("[ConnectionFailed] Exiting.");
                break ConnectionFailed;
            }
        };
    }
}