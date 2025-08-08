use bluest::Adapter;
use uuid::Uuid;
use futures_lite::StreamExt;
use anyhow::Result;

pub async fn receive_fling_key() -> Result<Vec<u8>> {
    // Set up the Bluetooth adapter
    let adapter = Adapter::default().await
        .ok_or_else(|| anyhow::anyhow!("No Bluetooth adapter available"))?;
    adapter.wait_available().await?;

    // Begin scanning for the fling service UUID
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

    // Discover services (this triggers connection if not already connected)
    let services = device.services().await?;
    let char_uuid = Uuid::parse_str("abcdef12-3456-7890-abcd-ef1234567890")?;
    let mut fling_char = None;

    for svc in services {
        if svc.uuid() == service_uuid {
            for ch in svc.characteristics().await? {
                if ch.uuid() == char_uuid {
                    fling_char = Some(ch);
                    break;
                }
            }
        }
    }

    let characteristic = fling_char.ok_or_else(|| anyhow::anyhow!("Fling characteristic not found"))?;

    // Read the key (connection is handled automatically)
    let key = characteristic.read().await?;
    println!("[Bluetooth][macOS] Fling key read ({} bytes)", key.len());

    // Explicit disconnect isn't provided; dropping the device will disconnect
    drop(device);

    Ok(key)
}
