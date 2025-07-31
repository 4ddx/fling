mod utils;
mod fsm;
use utils::cli::{Cli, Commands};
use clap::Parser;
use fsm::{sender_fsm, receiver_fsm};
/** fling send filepath.txt  || fling receive */

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Send {filepath } => {
            /***should invoke sender_fsm here*/
            println!("Sender Mode Enabled! FILE TO SEND: {} \nScanning for devices...", filepath);
            sender_fsm::start_sender_fsm(&filepath);
        }
        Commands::Receive => {
            /***should invoke receiver_fsm here*/
            println!("Receive Mode Enabled! \nListening for offers");
            receiver_fsm::start_receiver_fsm();
        }
    }
}
