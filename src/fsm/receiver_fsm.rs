use crate::bluetooth::{self, discovery::DeviceInfo};
use crate::tunnel;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
pub enum ReceiverState {
    Listening,
    Connecting(DeviceInfo),
    JoiningNetwork(String, String, String),
    ReadyToReceive,
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
                match adapter.wait_for_connection(10).await {
                    Ok(device_info) => {
                        println!("[Listening] Connected to sender: {}", device_info);
                        sleep(Duration::from_secs(5)).await; // Let MAC log settle
                        Connecting(device_info)
                    },
                    Err(e) => {
                        eprintln!("[Listening] Failed to wait for connection: {}", e);
                        ConnectionFailed
                    }
                }
            }

            Connecting(device_info) => {
                // Derive SSID & password deterministically based on *self*
                let hostname = whoami::hostname();
                let mac = adapter.get_own_address().await.unwrap();
                let suffix = &mac.replace(":", "").to_lowercase()[..4];
                let ssid = format!("fling-{}-{}", hostname, suffix);
                let password = format!("fling-{}!", suffix);                
                let ip_address = "192.168.49.1".to_string();

                JoiningNetwork(ssid, password, ip_address)
            }

            JoiningNetwork(ssid, password, ip_address) => {
                println!("[JoiningNetwork] Joining SSID {}...", ssid);
                match tunnel::connection::join_wifi_direct_network(&ssid, &password, "00:00:00:00:00:00").await {
                    Ok(_) => {
                        println!("[JoiningNetwork] Connected. Sender IP: {}", ip_address);
                        ReadyToReceive
                    },
                    Err(e) => {
                        eprintln!("[JoiningNetwork] Failed: {}", e);
                        ConnectionFailed
                    }
                }
            }

            ReadyToReceive => {
                println!("[ReadyToReceive] File receive starting...");
                Receiving
            }

            Receiving => {
                println!("[Receiving] Awaiting file over socket...");
                let save_path = "received_file";
                match tunnel::transfer::receive_file(save_path, "192.168.49.1").await {
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
            

            ReceiveSuccess => break ReceiveSuccess,
            ReceiveFailed => break ReceiveFailed,
            ConnectionFailed => break ConnectionFailed,
        };
    }
}
