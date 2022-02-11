#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod api;
mod cmds;
mod download;
mod gz;
mod rapid;
mod util;

#[derive(Parser)]
#[clap(author, version, about, long_about = "Rapid client")]
struct Args {
    #[clap(short, long)]
    root_folder: Option<PathBuf>,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check if SDP is fully downloaded
    CheckSdp { sdp: String },
    /// Download the specified rapid tag
    Download { tag: String },
    /// Download SDP
    DownloadSdp { sdp: String },
    /// Download the registry metadata
    DownloadRegistry,
    /// Download the repository metadata
    DownloadRepo { repo: Option<String> },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let root_folder = args.root_folder.unwrap_or_else(util::default_spring_dir);
    let rapid_store = rapid::rapid_store::RapidStore {
        root_folder: &root_folder,
    };

    match &args.command {
        Commands::CheckSdp { sdp } => {
            cmds::check_sdp(&rapid_store, sdp);
        }
        Commands::Download { tag } => {
            cmds::download(&rapid_store, tag).await;
        }
        Commands::DownloadSdp { sdp } => {
            cmds::download_sdp(&rapid_store, sdp).await;
        }
        Commands::DownloadRegistry => {
            cmds::download_registry(&rapid_store).await;
        }
        Commands::DownloadRepo { repo } => {
            cmds::download_repo(&rapid_store, repo.as_deref()).await;
        }
    }
}
