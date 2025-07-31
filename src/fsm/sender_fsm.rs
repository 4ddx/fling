use dialoguer::theme::ColorfulTheme;

#[derive(Debug)]
pub enum SenderState {
    Scanning,
    NoDevicesFound,
    Connecting(String),
    Sending,
    SendSuccess,
    SendFailed,
}

pub fn start_sender_fsm(filepath: &str) -> SenderState {
    use SenderState::*;

    let mut state = Scanning;

    loop {
        state = match state {
            Scanning => {
                println!("[Scanning] Searching for nearby receivers...");
                /*call bluetooth module to scan for devices */
                //let devices = bluetooth::scan_devices(Duration::from_secs(60)).await;
                let devices = vec!["iPhone 16 Pro Max::1E2EW::2343::423S::TW35", "Arch-1", "Test-Device-Alpha"];
                if devices.len() != 0 {
                    println!("[Scanning] Device found! Selecting device...");
                    let index = dialoguer::Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Select a device to connect to (default: 0)")
                        .items(&devices)
                        .default(0)
                        .interact().unwrap();
                    let chosen = devices[index].to_string();
                    println!("Connecting to {}", chosen);
                    SenderState::Connecting(chosen)
                } else {
                    println!("[Scanning] No receivers found.");
                    return SenderState::NoDevicesFound
                }
            }

            Connecting(s) => {
                println!("Waking up bluetooth module and sending discover packet to {}", s);
                SenderState::Sending
            }
            NoDevicesFound => {
                println!("[NoDevicesFound] Exiting sender FSM.");
                break SenderState::NoDevicesFound;
            }

            Sending => {
                println!("[Sending] Transferring file...");
                let success = simulate_file_send(filepath);
                if success {
                    println!("[Sending] File sent successfully!");
                    SendSuccess
                } else {
                    println!("[Sending] File transfer failed.");
                    SendFailed
                }
            }

            SendSuccess => {
                println!("[SendSuccess] Sender FSM complete.");
                break SendSuccess;
            }

            SendFailed => {
                println!("[SendFailed] Something went wrong. Try again.");
                break SendFailed;
            }
        }
    }
}

// ----- Mock Functions for Demo -----

// fn simulate_device_discovery() -> bool {
//     // Later: replace with actual broadcast + scan
//     true // pretend device was found
// }

fn simulate_file_send(_filepath: &str) -> bool {
    // Later: replace with actual file transfer logic
    true // pretend it was successful
}
