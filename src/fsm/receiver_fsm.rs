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
                let mac_fragment = &mac.replace(":", ":").to_lowercase();
                let suffix = &mac_fragment[mac_fragment.len()-4..];
                let password = crypto::crypto::generate_network_password(&hostname, &key);              
                let ssid = format!("fling-{}-{}-{}", hostname, suffix, &password[password.len()-2..]);
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

    let hostname_op = Command::new("scutil")
        .args(&["--get", "ComputerName"])
        .output()
        .expect("Failed to fetch hostname");

    let name = if hostname_op.status.success() {
        let name = String::from_utf8_lossy(&hostname_op.stdout).trim().to_string();
        name
    }else {
        let err_m = String::from_utf8_lossy(&hostname_op.stderr).trim().to_string();
        eprintln!("Failed to get computer name: {}", err_m);
        err_m
    };
    name
}

#[cfg(target_os = "linux")]
fn get_hostname() -> String {
    whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string())
}
