use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name="fling", author, version, about="Airdrop for *Nix")]
pub struct Cli{
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Send {
        #[arg(value_name="FILE")]
        filepath: String,
    },
    Receive,
}

