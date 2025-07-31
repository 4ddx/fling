use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const PORT: u16 = 8080;
const BUFFER_SIZE: usize = 64 * 1024; // 64KB buffer

/// Sender: Listen for a TCP connection and send the file to the first client that connects
pub async fn send_file(filepath: &str) -> Result<(), String> {
    let listener = TcpListener::bind(("0.0.0.0", PORT))
        .await
        .map_err(|e| format!("[Sender] Failed to bind: {}", e))?;

    println!("[Sender] Awaiting receiver connection on port {}...", PORT);
    let (mut socket, addr) = listener.accept().await.map_err(|e| format!("[Sender] Accept failed: {}", e))?;
    println!("[Sender] Receiver connected from {}", addr);

    let mut file = File::open(filepath)
        .await
        .map_err(|e| format!("[Sender] Could not open file: {}", e))?;

    let mut buffer = vec![0u8; BUFFER_SIZE];
    loop {
        let n = file.read(&mut buffer).await.map_err(|e| format!("[Sender] Read error: {}", e))?;
        if n == 0 {
            break;
        }
        socket.write_all(&buffer[..n]).await.map_err(|e| format!("[Sender] Write error: {}", e))?;
    }

    println!("[Sender] File transfer complete.");
    Ok(())
}

/// Receiver: Connect to sender and write incoming data to a file
pub async fn receive_file(save_path: &str, sender_ip: &str) -> Result<(), String> {
    let addr = format!("{}:{}", sender_ip, PORT);
    let mut socket = TcpStream::connect(&addr)
        .await
        .map_err(|e| format!("[Receiver] Connection failed: {}", e))?;

    println!("[Receiver] Connected to sender at {}", addr);

    let mut file = File::create(save_path)
        .await
        .map_err(|e| format!("[Receiver] Failed to create file: {}", e))?;

    let mut buffer = vec![0u8; BUFFER_SIZE];
    loop {
        let n = socket.read(&mut buffer).await.map_err(|e| format!("[Receiver] Read error: {}", e))?;
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n]).await.map_err(|e| format!("[Receiver] Write error: {}", e))?;
    }

    println!("[Receiver] File received successfully.");
    Ok(())
}