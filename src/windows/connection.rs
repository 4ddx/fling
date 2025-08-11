use std::{process::Command, time::Duration};
use tokio::time::sleep;

/// Launches a Wi-Fi Direct access point using hostapd and sets up DHCP
pub async fn create_wifi_direct_network(ssid: &str, password: &str) -> Result<String, String> {}

/// Waits up to 30 seconds for a client to join the AP
pub async fn wait_for_receiver() -> Result<String, String> {}

pub async fn cleanup_wifi() {}

///Joins a Full AP network controlled by sender
pub fn join_wifi_direct_network(ssid: &str, password: &str) -> bool {}


