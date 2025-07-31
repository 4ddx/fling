use std::fs;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

/// Launches a Wi-Fi Direct access point using hostapd and sets up DHCP
pub async fn create_wifi_direct_network(ssid: &str, password: &str) -> Result<String, String> {
    let iface = "wlan0"; // change if needed

    // 1. Write hostapd.conf
    let hostapd_conf = format!(
        "interface={}
driver=nl80211
ssid={}
hw_mode=g
channel=6
auth_algs=1
wmm_enabled=1
wpa=2
wpa_passphrase={}
wpa_key_mgmt=WPA-PSK
rsn_pairwise=CCMP
",
        iface, ssid, password
    );

    fs::write("/tmp/hostapd.conf", hostapd_conf)
        .map_err(|e| format!("Failed to write hostapd.conf: {}", e))?;

    // 2. Bring interface up and assign static IP
    run("ip", &["link", "set", iface, "up"])?;
    run("ip", &["addr", "flush", "dev", iface])?;
    run("ip", &["addr", "add", "192.168.49.1/24", "dev", iface])?;

    // 3. Launch hostapd
    let _ = Command::new("pkill").arg("hostapd").output();
    let _hostapd = Command::new("hostapd")
        .arg("/tmp/hostapd.conf")
        .spawn()
        .map_err(|e| format!("Failed to start hostapd: {}", e))?;

    // 4. Launch simple DHCP server
    let _ = Command::new("pkill").arg("udhcpd").output();
    fs::write(
        "/tmp/udhcpd.conf",
        "start 192.168.49.10\nend 192.168.49.50\ninterface wlan0\noption subnet 255.255.255.0\n",
    )
    .map_err(|e| format!("Failed to write udhcpd config: {}", e))?;

    run("udhcpd", &["/tmp/udhcpd.conf"])?;

    // Let things settle
    sleep(Duration::from_secs(2)).await;

    Ok("192.168.49.1".into())
}

/// Waits up to 30 seconds for a client to join the AP
pub async fn wait_for_receiver() -> Result<(), String> {
    for _ in 0..30 {
        if let Ok(output) = Command::new("iw")
            .args(["dev", "wlan0", "station", "dump"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Station") {
                return Ok(());
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
    Err("No clients joined within timeout".into())
}

///Joins a Full AP network controlled by sender
pub async fn join_wifi_direct_network(ssid: &str, password: &str, _mac_hint: &str) -> Result<(), String> {
    use std::process::Command;

    // Replace with nmcli or wpa_supplicant depending on the platform
    let output = Command::new("nmcli")
        .args(&["dev", "wifi", "connect", ssid, "password", password])
        .output()
        .map_err(|e| format!("Failed to launch nmcli: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Failed to connect to WiFi: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}


fn run(cmd: &str, args: &[&str]) -> Result<(), String> {
    Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {}: {}", cmd, e))?;
    Ok(())
}
