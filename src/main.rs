mod bluetooth;

#[tokio::main]
async fn main() {
    let instance = bluetooth::initialize().await; 
    let descriptors = match bluetooth::scan_devices(&instance).await {
        Ok(descriptors) => descriptors,
        Err(_) => Vec::new()
    };

    for descriptor in descriptors {
        println!("Descriptor: {}", descriptor.name);
    }
}

