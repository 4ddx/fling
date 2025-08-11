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

pub async fn send_file(filepath: &str) -> Result<(), String> {}

pub async fn receive_file(output_dir: &str) -> Result<(), String> {}