mod bluetooth;
mod fsm;
mod utils;
use clap::Parser;
use fsm::{receiver_fsm, sender_fsm};
use utils::cli::{Cli, Commands};
mod tunnel;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Send { filepath } => {
            println!("Sender Mode Enabled!\nFile to send: {}", filepath);
            sender_fsm::start_sender_fsm(&filepath).await;
        }
        Commands::Receive => {
            println!("Receiver Mode Enabled!\nListening for offers...");
             receiver_fsm::start_receiver_fsm().await;
        }
    }
}
