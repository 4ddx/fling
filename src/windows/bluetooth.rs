use std::error::Error;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Clone, Debug)]


impl AdapterController {
    pub async fn initialize() -> Result<Self> {}

    pub async fn scan_devices(
        &self,
        timeout_secs: Duration,
    ) -> Result<Vec<>, Box<dyn std::error::Error>> {}

    pub async fn serve_gatt(
        &self,
        key_data: Vec<u8>,
        expected_mac: String,
    ) -> Result<(), Box<dyn std::error::Error>> {}
}

pub async fn receive_fling_key() -> Result<Vec<u8>, Box<dyn Error>> {
}

pub fn get_bluetooth_mac() -> Option<String> {

}