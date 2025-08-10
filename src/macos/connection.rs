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
    while start.elapsed() < timeout {
        let output = Command::new("networksetup")
            .args(&["-setairportnetwork", interface, ssid, password])
            .output();
        
    match output {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            eprintln!("Command failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        Err(e) => {
            eprintln!("Failed to run command: {}", e);
        }
        thread_sleep(Duration::from_secs(2));
    }
        true
}
