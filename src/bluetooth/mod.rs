use std::pin::Pin;
use std::time::Duration;

use bluer::{Adapter, Address, Result };
use bluer::AdapterEvent;
use futures::{pin_mut, Stream, StreamExt};

pub struct FlingInstance {
    adapter: Adapter
}

pub struct DeviceDescriptor {
    pub name: String,
    pub address: String
}

pub impl DeviceDescriptor {
    async fn new(device: &bluer::Device) -> Result<DeviceDescriptor> {
        let descriptor = DeviceDescriptor { 
            name: device.name().await?.unwrap_or_default(),
            address: device.address().to_string()
        };

        Ok(descriptor)
    }
}


pub async fn initialize() -> FlingInstance {
    let session = bluer::Session::new().await.unwrap();
    let adapter = session.default_adapter().await.unwrap();
    adapter.set_powered(true).await.expect("Failed to power on");

    FlingInstance { adapter: adapter }
}

async fn query_device_details(adapter: &Adapter, addr: Address) -> bluer::Result<String> {
    let device = adapter.device(addr)?;
    let name: String = match device.name().await? {
        Some(name) => name,
        None => "NULL".to_string()
    };

    Ok(name)
}

pub async fn scan_devices(instance: &FlingInstance) -> Result<Vec<DeviceDescriptor>> {
    let discovery_session = instance.adapter.discover_devices().await?;
    let mut result: Vec<DeviceDescriptor> = Vec::new();
    pin_mut!(discovery_session);
    
    let res = tokio::time::timeout(
        Duration::from_secs(20), 
        discovery_loop(&instance, &mut result, &mut discovery_session)
    ).await;

    println!("Finished scanning for devices");
    Ok(result)
}

async fn discovery_loop(
    instance: &FlingInstance, 
    device_descriptors: &mut Vec<DeviceDescriptor>,
    discovery_session: &mut Pin<&mut impl Stream<Item = AdapterEvent>>,
) -> Result<()> {
    loop {
        tokio::select! {
            Some(device_event) = discovery_session.next() => {
                match device_event {
                    AdapterEvent::DeviceAdded(addr) => {
                        let name = query_device_details(&instance.adapter, addr).await?;
                        // println!("Added Device: {}", name);
                        device_descriptors.push(DeviceDescriptor { name: name });
                    }

                    AdapterEvent::DeviceRemoved(addr) => {
                        let name = query_device_details(&instance.adapter, addr).await?;
                        // println!("Removed Device: {}", name);
                        device_descriptors.pop_if(|item| {
                            return item.name == name;
                        });
                    }

                    _ => { return Ok(())}
                }   
            }
        }
    }

}

