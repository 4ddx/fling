use std::{process::Command, time::Duration};
use anyhow::Result;
use bluest::Adapter;
use futures_lite::StreamExt;
use uuid::Uuid;

pub fn get_bluetooth_mac() -> Option<String> {
    let output = Command::new("system_profiler")
        .arg("SPBluetoothDataType")
        .output()
        .ok()?
        .stdout;
    String::from_utf8_lossy(&output)
        .lines()
        .find_map(|line| {
        line.trim()
            .strip_prefix("Address: ")
            .map(|s| s.trim().to_string())
    })
}
pub async fn receive_fling_key() -> Result<Vec<u8>> {

    let adapter = Adapter::default()
        .await
        .ok_or_else(|| anyhow::anyhow!("No Bluetooth adapter available"))?;
    adapter.wait_available().await?;

    let mut scan = adapter.scan(&[]).await?;
    let service_uuid = Uuid::parse_str("12345678-1234-5678-1234-56789abcdef0")?;

    println!("Scanning for fling sender...");
    let device = loop {
        if let Some(discovered) = scan.next().await {
            if discovered.adv_data.services.contains(&service_uuid) {
                println!("Found fling sender!");
                break discovered.device;
            }
        }
    };
    drop(scan);
    tokio::time::sleep(Duration::from_secs(2)).await;

    adapter.connect_device(&device).await?;


    let mut retries = 0;
    while !device.is_connected().await && retries<10 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        retries += 1;
    }

    if !device.is_connected().await {
        return Err(anyhow::anyhow!("Failed to establish connection"));
    }

    device.discover_services().await?;

    tokio::time::sleep(Duration::from_millis(500)).await;

    let services = device.services().await?;
    let char_uuid = Uuid::parse_str("abcdef12-3456-7890-abcd-ef1234567890")?;
    let mut fling_char = None;

    for svc in services {
        let svc_uuid = svc.uuid();
        let _ = svc.discover_characteristics().await;
        let characteristics = svc.characteristics().await?;
        for ch in characteristics { 
            let ch_uuid = ch.uuid();
            if ch_uuid == char_uuid {
                println!("Found fling char.!");
                fling_char = Some(ch);
                break;
            }
        }
        if svc_uuid == service_uuid && fling_char.is_none() {
            println!("Found fling svc, but char. was not found in it");
        }
    }
    let characteristic =
        fling_char.ok_or_else(|| anyhow::anyhow!("Fling characteristic not found"))?;


    let key = characteristic.read().await?;
    println!("[Bluetooth][macOS] Fling key read ({} bytes)", key.len());

    adapter.disconnect_device(&device.clone()).await?;
    Ok(key)
}