use crate::{bluetooth, crypto, macos};
use crate::macos::bluetooth::receive_fling_key;
// use crate::tunnel::{self, connection};
use tokio::time::{sleep, Duration};

#[derive(Debug)]
pub enum ReceiverState {
    Listening,
    Connecting(Vec<u8>),
    JoiningNetwork(String, String, String),
    Receiving,
    ReceiveSuccess,
    ReceiveFailed,
    ConnectionFailed,
}

pub async fn start_receiver_fsm() -> ReceiverState {
    use ReceiverState::*;

    println!("[Listening] Initializing Bluetooth...");
    let adapter = match bluetooth::discovery::AdapterController::initialize().await {
        Ok(ctrl) => ctrl,
        Err(e) => {
            eprintln!("[Listening] Failed to init adapter: {}", e);
            return ConnectionFailed;
        }
    };

    if let Err(e) = adapter.make_discoverable().await {
        eprintln!("[Listening] Failed to become discoverable: {}", e);
        return ConnectionFailed;
    }

    let mut state = Listening;

    loop {
        state = match state {
            Listening => {
                println!("[Listening] Waiting for Bluetooth connection...");
                match receive_fling_key().await {
                    Ok(key) => {
                        let key_str = String::from_utf8_lossy(&key);
                        println!("[Listening] Connected to sender. Received Key: {}", key_str);
                
                        sleep(Duration::from_secs(5)).await;
                        Connecting(key)
                    },
                    Err(e) => {
                        eprintln!("[Listening] Failed to wait for connection: {}", e);
                        ConnectionFailed
                    }
                }
                
            }

            Connecting(key) => {
                // Derive SSID & password deterministically based on *self*
                #[allow(deprecated)]
                let hostname = whoami::hostname();
                let mac = adapter.get_own_address().await.unwrap();
                let suffix = &mac.replace(":", "").to_lowercase()[..4];
                let ssid = format!("fling-{}-{}", hostname, suffix);
                let password = crypto::crypto::generate_network_password(&hostname, &key);              
                let ip_address = "10.42.0.1".to_string();

                JoiningNetwork(ssid, password, ip_address)
            }

            JoiningNetwork(ssid, password, _ip_address) => {
                println!("[JoiningNetwork] Joining SSID {}...", ssid);
                if macos::connection::join_wifi_direct_network(&ssid, &password) == true {
                    
                        Receiving
                    }else {
                        ConnectionFailed
                    }
                }

            Receiving => {
                println!("[Receiving] Awaiting file over socket...");
                let save_path = "Rec_Folder";
                match macos::transfer::receive_file(save_path).await {
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
