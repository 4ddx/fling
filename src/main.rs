mod bluetooth;
mod fsm;
mod utils;
use clap::Parser;
use utils::cli::{Cli, Commands};
mod tunnel;
mod linux;
mod macos;
mod crypto;
use tokio::signal;
use std::sync::Arc;
use tokio::sync::Notify;

#[tokio::main]
async fn main() {
    let shutdown_notify = Arc::new(Notify::new());
    let shutdown_handle = shutdown_notify.clone();

    tokio::spawn(async move {
        if signal::ctrl_c().await.is_ok() {
            println!("\n[Signal] Caught Ctrl+C! Cleaning up...");
            linux::connection::cleanup_wifi().await;
            shutdown_handle.notify_one();
        }
    });

    let cli = Cli::parse();

    match cli.command {
        Commands::Send { filepath } => {
            println!("Sender Mode Enabled!\nFile to send: {}", filepath);
            if std::env::consts::OS=="macos" {
                println!("Sending from a MAC is not currently supported.\nSee README.md for more details.");
                std::process::exit(1);
        }
            fsm::sender_fsm::start_sender_fsm(&filepath).await;
        }
        Commands::Receive => {
            println!("Receiver Mode Enabled!\nListening for offers...");
            fsm::receiver_fsm::start_receiver_fsm().await;
    }
}
}

