use std::{net::TcpStream, process::Command, time::Duration};
use tokio::time::sleep;
use std::time::Instant;
use std::thread::sleep as thread_sleep;

pub async fn wait_for_ip() -> Result<(), String> {
    for _attempt in 1..=30{
        let output = Command::new("ifconfig")
            .arg("en0")
            .output()
            .map_err(|e| format!("Failed to run ifconfig: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("inet 10.42.0.") {
            println!("[Receiver] Got IP on en0");
            return Ok(());
        }
        sleep(Duration::from_millis(1000)).await;
    }
    Err("Timed out waiting for IP on en0".to_string())
}
pub fn wait_for_port(host: &str, port: u16) -> TcpStream {
    let addr = format!("{}:{}", host, port);
    loop {
        match TcpStream::connect(&addr) {
            Ok(stream) => {
                println!("[Receiver]✅ Connected to {}", addr);
                return stream;
            }
            Err(_) => {
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                let _ = sleep(Duration::from_millis(500));
            }
        }
    }
}
pub fn join_wifi_direct_network(ssid: &str, password: &str) -> bool {
    let interface = "en0";
    let timeout = Duration::from_secs(20);
    let start_time = Instant::now();

    println!("[JoiningNetwork] Trying to connect to Wi-Fi: {}", ssid);

        let output = Command::new("networksetup")
            .args(&["-setairportnetwork", interface, ssid, password])
            .output();

        if let Ok(output) = output {
            if !output.status.success() {
                eprintln!("[Error] networksetup failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        } else {
            eprintln!("[Error] Failed to execute networksetup command.");
        }

        thread_sleep(Duration::from_secs(2));
        while start_time.elapsed() < timeout {
        let verify_output = Command::new("networksetup")
            .args(&["-getairportnetwork", interface])
            .output();

        if let Ok(verify_output) = verify_output {
            if verify_output.status.success() {
                let output_str = String::from_utf8_lossy(&verify_output.stdout);
                if output_str.contains(ssid) {
                    println!("[Success] Successfully connected to Wi-Fi network: {}", ssid);
                    return true;
                } else {
                    continue;
                }
            } else {
                eprintln!("[Error] Failed to get current Wi-Fi network: {}", String::from_utf8_lossy(&verify_output.stderr));
            }
        }
        thread_sleep(Duration::from_secs(2));
    }

    eprintln!("[Timeout] Could not connect to Wi-Fi network '{}' within 20 seconds.", ssid);
    false
}
