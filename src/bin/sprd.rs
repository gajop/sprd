#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use sprd::{api::DownloadOptions, cmds, rapid};

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

    let rapid_store = match args.root_folder {
        Some(path) => rapid::rapid_store::RapidStore::new(path),
        None => rapid::rapid_store::RapidStore::default(),
    };

    match &args.command {
        Commands::CheckSdp { sdp } => {
            cmds::check_sdp(&rapid_store, sdp);
        }
        Commands::Download { tag } => {
            cmds::download(&rapid_store, &DownloadOptions::default(), tag).await;
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
