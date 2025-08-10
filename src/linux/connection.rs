use std::{process::Command, time::Duration};
use tokio::time::sleep;

/// Launches a Wi-Fi Direct access point using hostapd and sets up DHCP
pub async fn create_wifi_direct_network(ssid: &str, password: &str) -> Result<String, String> {
    let _iface = "wlan0";

    // Kill any existing hotspot connections
    let _ = Command::new("nmcli")
        .args(&["con", "down", "Hotspot"])
        .output(); // ignore failure

    let _ = Command::new("nmcli").args(&["dev", "disconnect", "wlan0"]).output();

    let cmd = Command::new("nmcli")
        .args(&[
            "device", "wifi", "hotspot",
            "ifname", "wlan0",
            "band", "a",
            "channel", "149",
            "ssid", &ssid,
            "password", &password,
        ])
        .output()
        .map_err(|e| format!("Failed to spawn nmcli: {}", e))?;
    if !cmd.status.success() {
        return Err(format!(
            "Hotspot creation failed: {}",
            String::from_utf8_lossy(&cmd.stderr)
        ));
    }

    // Optional: give it 2 seconds to settle
    sleep(std::time::Duration::from_secs(2)).await;

    // Done. Return the static gateway IP that nmcli usually assigns
    Ok("10.42.0.1".to_string()) // default IP used by nmcli hotspot
}

/// Waits up to 30 seconds for a client to join the AP
pub async fn wait_for_receiver() -> Result<String, String> {
    for _ in 0..30 {
        // Check ARP table
        let arp = Command::new("ip")
            .args(&["neigh"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default();

        for line in arp.lines() {
            if line.contains("10.42.0.") && line.contains("lladdr") && !line.contains("FAILED") {
                let ip = line.split_whitespace().next().unwrap_or_default().to_string();
                println!("[ReceiverConnected] IP: {}", ip);
                return Ok(ip);
            }
        }

        sleep(Duration::from_secs(1)).await;
    }

    Err("No clients joined within timeout".into())
}

pub async fn cleanup_wifi() {
    // Kill hotspot
    let _ = Command::new("nmcli")
        .args(&["con", "down", "Hotspot"])
        .output();

    // Restart NetworkManager service
    let _ = Command::new("systemctl")
        .args(&["restart", "NetworkManager"])
        .output();

    // Optionally re-enable Wi-Fi
    let _ = Command::new("nmcli")
        .args(&["radio", "wifi", "on"])
        .output();

    println!("[Cleanup] Wi-Fi state cleaned up and reset.");
}
///Joins a Full AP network controlled by sender
pub fn join_wifi_direct_network(ssid: &str, password: &str) -> bool {
    use std::process::Command;

    let output = Command::new("nmcli")
        .args(&["dev", "wifi", "connect", ssid, "password", password])
        .output();

    match output {
        Ok(o) if o.status.success() => true,
        _ => false,
    }
}


