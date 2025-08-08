use crate::macos::connection::{wait_for_ip, wait_for_port};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::time::Instant;
use tokio::
    process::Command
;
const BUF_SIZE: usize = 1024 * 1024;
const PORT: u16 = 8080;

pub async fn receive_file(save_dir: &str) -> Result<(), String> {
    println!("[Receiver] Connecting to sender at 10.42.0.1:8080");

    let temp_tar = "/tmp/fling_recv.tar.gz";
    wait_for_ip().await?;

    let stream = tokio::task::spawn_blocking(|| wait_for_port("10.42.0.1", PORT))
        .await
        .map_err(|e| format!("Task join error: {}", e))?;
    let mut reader = std::io::BufReader::with_capacity(BUF_SIZE, stream);
    let file = File::create(temp_tar).map_err(|e| format!("File error: {}", e))?;
    let mut writer = BufWriter::with_capacity(BUF_SIZE, file);

    let mut buffer = vec![0u8; BUF_SIZE];
    let mut total_bytes = 0;
    let start = Instant::now();

    loop {
        let n = reader
            .read(&mut buffer)
            .map_err(|e| format!("Read error: {}", e))?;
        if n == 0 {
            break;
        }
        writer
            .write_all(&buffer[..n])
            .map_err(|e| format!("Write error: {}", e))?;
        total_bytes += n;
    }
    writer.flush().unwrap();

    let elapsed = start.elapsed().as_secs_f64();
    let mbps = (total_bytes as f64 * 8.0) / (elapsed * 1_000_000.0);
    println!(
        "[Receiver] ✅ Received {:.2} MB in {:.2}s ({:.2} Mbps)",
        total_bytes as f64 / 1_000_000.0,
        elapsed,
        mbps
    );

    if !Path::new(save_dir).exists() {
        std::fs::create_dir_all(save_dir).map_err(|e| format!("Create dir failed: {}", e))?;
    }

    let output = Command::new("tar")
        .args(&["-xzf", temp_tar, "-C", save_dir])
        .output()
        .await
        .map_err(|e| format!("Untar failed: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Untar error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let size = Command::new("du")
        .args(&["-sm", save_dir])
        .output()
        .await
        .map_err(|e| format!("Failed to run du: {}", e))?;

    let stdout = String::from_utf8_lossy(&size.stdout);
    if let Some(first) = stdout.split_whitespace().next() {
        if let Ok(actual_mb) = first.parse::<f64>() {
            let mbps = (actual_mb * 8.0) / elapsed;
            println!("[Receiver] Effective speed: {:.2} Mbps", mbps);
        }
    }

    println!("[Receiver] ✅ Extracted to '{}'", save_dir);
    Ok(())
}