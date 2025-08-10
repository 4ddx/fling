#![cfg(target_os="linux")]
use dialoguer::{Select, theme::ColorfulTheme};
use std::time::Duration;

use crate::bluetooth;
use crate::crypto;
use crate::tunnel;
#[derive(Debug)]
pub enum SenderState {
    Scanning,
    Connecting(bluetooth::discovery::DeviceInfo),
    ServingGatt(bluetooth::discovery::DeviceInfo),
    StartingHotspot(bluetooth::discovery::DeviceInfo, String),
    WaitingForJoin(bluetooth::discovery::DeviceInfo),
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
                let devices = match adapter.scan_devices(Duration::from_secs(5)).await {
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
                ServingGatt(device_info)
            }
            ServingGatt(device_info) => {
                println!("[GATT] Starting GATT server for key exchange...");

                let device_address = match device_info.address.parse::<bluer::Address>() {
                    Ok(addr) => addr,
                    Err(e) => {
                        eprintln!("[GATT] Invalid MAC address format: {}", e);
                        return ConnectionFailed;
                    }
                };
                let crypto_key = crypto::crypto::generate_encryption_key();
                let net_pass =
                    crypto::crypto::generate_network_password(&device_info.name, &crypto_key);

                match adapter.serve_gatt(crypto_key, device_address).await {
                    Ok(_) => {
                        StartingHotspot(device_info, net_pass)
                    }
                    Err(e) => {
                        eprintln!("[GATT] Failed to start GATT server: {}", e);
                        ConnectionFailed
                    }
                }
            }

            StartingHotspot(device_info, net_pass) => {
                //Use receiver's hostname + BT MAC to create deterministic SSID
                let hostname = device_info.name.clone();
                let mac_fragment = device_info.address.replace(":", "").to_lowercase();
                let suffix = &mac_fragment[mac_fragment.len() - 4..];
                let ssid = format!("fling-{}-{}-{}", hostname, suffix, &net_pass[net_pass.len()-2..]);
                match tunnel::connection::create_wifi_direct_network(&ssid, &net_pass).await {
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
                tunnel::connection::cleanup_wifi().await;
                break SendSuccess;
            }

            SendFailed => {
                println!("[❌] Transfer failed.");
                tunnel::connection::cleanup_wifi().await;
                break SendFailed;
            }

            NoDevicesFound => {
                println!("[NoDevicesFound] Exiting.");
                tunnel::connection::cleanup_wifi().await;
                break NoDevicesFound;
            }

            ConnectionFailed => {
                println!("[ConnectionFailed] Exiting.");
                tunnel::connection::cleanup_wifi().await;
                break ConnectionFailed;
            }
        };
    }
}
