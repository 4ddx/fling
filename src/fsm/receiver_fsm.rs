use crate::{bluetooth, crypto, tunnel};

#[derive(Debug)]
pub enum ReceiverState {
    Listening,
    Connecting(Vec<u8>),
    JoiningNetwork(String, String),
    Receiving,
    ReceiveSuccess,
    ReceiveFailed,
    ConnectionFailed,
}

pub async fn start_receiver_fsm() -> ReceiverState {
    use ReceiverState::*;

    let mut state = Listening;

    loop {
        state = match state {
            Listening => {
                println!("[Listening] Waiting for Bluetooth connection...");
                match bluetooth::discovery::receive_fling_key().await {
                    Ok(key) => {
                        let key_str = String::from_utf8_lossy(&key);
                        println!("[Listening] Connected to sender. Received Key: {}", key_str);
                        Connecting(key)
                    },
                    Err(e) => {
                        eprintln!("[Listening] Failed to wait for connection: {}", e);
                        ConnectionFailed
                    }
                }
                
            }

            Connecting(key) => {
                #[allow(deprecated)]
                let hostname = get_hostname();
                let mac = bluetooth::discovery::get_bluetooth_mac().unwrap();
                let suffix = &mac.replace(":", "").to_lowercase()[..4];
                let ssid = format!("fling-{}-{}", hostname, suffix);
                let password = crypto::crypto::generate_network_password(&hostname, &key);              

                JoiningNetwork(ssid, password)
            }

            JoiningNetwork(ssid, password) => {
                println!("[JoiningNetwork] Joining SSID {}...", ssid);
                if tunnel::connection::join_wifi_direct_network(&ssid, &password) == true {
                        Receiving
                    }else {
                        ConnectionFailed
                    }
                }

            Receiving => {
                println!("[Receiving] Awaiting file over socket...");
                let save_path = "Rec_Folder";
                match tunnel::transfer::receive_file(save_path).await {
                    Ok(_) => {
                        println!("[Receiving] File transfer complete!");
                        ReceiveSuccess
                    },
                    Err(e) => {
                        eprintln!("[Receiving] Transfer failed: {}", e);
                        ReceiveFailed
                    }
                }
            }
            

            ReceiveSuccess => {
                break ReceiveSuccess
            },
            ReceiveFailed => {
                break ReceiveFailed
            },
            ConnectionFailed => {
                break ConnectionFailed
            },
        };
    }
}

#[cfg(target_os = "macos")]
fn get_hostname() -> String {
    use std::process::Command;

    let output = Command::new("scutil")
        .args(&["--get", "ComputerName"])
        .output()
        .expect("Failed to fetch hostname");

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        eprintln!(
            "Failed to get computer name: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        "unknown".to_string()
    }
}

#[cfg(target_os = "linux")]
fn get_hostname() -> String {
    whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string())
}
