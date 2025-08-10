use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::process::Stdio;
use std::time::Instant;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    process::Command,
    time::{Duration},
};

const PORT: u16 = 8080;
const BUF_SIZE: usize = 1024 * 1024;

pub async fn send_file(filepath: &str) -> Result<(), String> {
    let tar_path = "/tmp/fling_tmp.tar.gz";
    if Path::new(tar_path).exists() {
        let _ = tokio::fs::remove_file(tar_path).await;
    }

    let listener = TcpListener::bind(("0.0.0.0", PORT))
        .await
        .map_err(|e| format!("Bind failed: {}", e))?;

    let output = Command::new("tar")
        .args(&["-I", "zstd", "-cf", tar_path, filepath])
        .output()
        .await
        .map_err(|e| format!("Tar failed: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Tar error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    println!("[Sender] Waiting for receiver on port {}...", PORT);
    let (mut socket, addr) = listener
        .accept()
        .await
        .map_err(|e| format!("Accept failed: {}", e))?;
    println!("[Sender] Connected to {}", addr);

    let tar_metadata = tokio::fs::metadata(tar_path)
        .await
        .map_err(|e| format!("Metadata failed: {}", e))?;
    let total_size = tar_metadata.len();

    let file = File::open(tar_path)
        .await
        .map_err(|e| format!("File open failed: {}", e))?;
    let mut reader = BufReader::with_capacity(BUF_SIZE, file);
    // Setup progress bar
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec})",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    let mut buffer = vec![0u8; BUF_SIZE];
    let mut total_bytes = 0u64;
    let start = Instant::now();

    loop {
        let n = reader
            .read(&mut buffer)
            .await
            .map_err(|e| format!("Read failed: {}", e))?;
        if n == 0 {
            break;
        }
        socket
            .write_all(&buffer[..n])
            .await
            .map_err(|e| format!("Write failed: {}", e))?;
        total_bytes += n as u64;
        pb.set_position(total_bytes);
    }

    pb.finish_with_message("✅ Transfer complete");

    let elapsed = start.elapsed().as_secs_f64();
    let mbps = (total_bytes as f64 * 8.0) / (elapsed * 1_000_000.0);
    println!(
        "[Sender] ✅ Sent {:.2} MB in {:.2}s ({:.2} Mbps)",
        total_bytes as f64 / 1_000_000.0,
        elapsed,
        mbps
    );

    Ok(())
}

pub async fn receive_file(output_dir: &str) -> Result<(), String> {
    use tokio::{io::BufWriter, net::TcpStream, time::sleep};

    let tar_path = "/tmp/fling_received_tmp.tar.gz";
    if Path::new(tar_path).exists() {
        let _ = tokio::fs::remove_file(tar_path).await;
    }

    println!("[Receiver] Connecting to sender at 10.42.0.1:8080");
    sleep(Duration::from_secs(2)).await;

    let stream = TcpStream::connect("10.42.0.1:8080")
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;
    println!("[Receiver] Connected to sender!");

    let file = File::create(tar_path)
        .await
        .map_err(|e| format!("File error: {}", e))?;
    let mut writer = BufWriter::with_capacity(BUF_SIZE, file);
    let mut reader = tokio::io::BufReader::with_capacity(BUF_SIZE, stream);
    let mut buffer = vec![0u8; BUF_SIZE];

    let mut total_bytes = 0u64;
    let start = Instant::now();
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {spinner} {bytes} received")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
    );
    pb.enable_steady_tick(Duration::from_millis(100));

    loop {
        let n = reader
            .read(&mut buffer)
            .await
            .map_err(|e| format!("Read error: {}", e))?;
        if n == 0 {
            break;
        }
        writer
            .write_all(&buffer[..n])
            .await
            .map_err(|e| format!("Write error: {}", e))?;
        total_bytes += n as u64;
        pb.set_position(total_bytes);
    }

    writer.flush().await.unwrap();
    pb.finish_with_message("✅ Tarball received");

    let elapsed = start.elapsed().as_secs_f64();
    let mbps = (total_bytes as f64 * 8.0) / (elapsed * 1_000_000.0);
    println!(
        "[Receiver] ✅ Received {:.2} MB in {:.2}s ({:.2} Mbps)",
        total_bytes as f64 / 1_000_000.0,
        elapsed,
        mbps
    );

    // Now unpack the tarball
    let untar_start = Instant::now();
    let output = Command::new("tar")
        .args(&["-xzf", tar_path, "-C", output_dir])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Untar failed: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Untar error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let untar_elapsed = untar_start.elapsed().as_secs_f64();

    // Get expanded size
    let du_out = Command::new("du")
        .args(&["-sm", output_dir])
        .output()
        .await
        .map_err(|e| format!("du command failed: {}", e))?;

    let du_stdout = String::from_utf8_lossy(&du_out.stdout);
    let real_size_mb = du_stdout
        .split_whitespace()
        .next()
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);

    let real_mbps = (real_size_mb * 8.0) / (elapsed + untar_elapsed);
    println!(
        "[Receiver] 📦 Extracted {:.2} MB in {:.2}s (Effective {:.2} Mbps)",
        real_size_mb,
        untar_elapsed,
        real_mbps
    );

    Ok(())
}