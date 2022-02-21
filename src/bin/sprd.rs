#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

use std::{path::PathBuf, sync::Arc};

use clap::{ArgEnum, Parser, Subcommand};
use output::{interactive::InteractiveOutput, json::JsonOutput};
use sprd::{
    api::{DownloadOptions, MetadataSource},
    cmds,
    event::{PrintOutput, SilentOutput},
    rapid,
};

use atty::Stream;

mod output;

#[derive(Parser)]
#[clap(author, version, about, long_about = "Rapid client")]
struct Args {
    #[clap(short, long)]
    root_folder: Option<PathBuf>,

    #[clap(short, long, arg_enum, default_value_t = PrintType::Auto)]
    print: PrintType,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum PrintType {
    Auto,
    Silent,
    Print,
    Json,
    Interactive,
}

#[derive(Subcommand)]
enum Commands {
    /// Download the specified rapid tag
    Download { tag: String },
    /// Download SDP
    DownloadSdp { sdp: String },
    /// Download the registry metadata
    DownloadRegistry,
    /// Download the repository metadata
    DownloadRepo { repo: Option<String> },

    /// Check if SDP is fully downloaded
    CheckSdp { sdp: String },
    /// Validate by fullname
    Validate { fullname: String },
    /// Fix
    Fix { fullname: String },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let rapid_store = match args.root_folder {
        Some(path) => rapid::rapid_store::RapidStore::new(path),
        None => rapid::rapid_store::RapidStore::default(),
    };

    let mut opts = DownloadOptions {
        print: match args.print {
            PrintType::Silent => Arc::new(Box::new(SilentOutput {})),
            PrintType::Auto => {
                if atty::is(Stream::Stdout) {
                    Arc::new(Box::new(InteractiveOutput::new()))
                } else {
                    Arc::new(Box::new(PrintOutput {}))
                }
            }
            PrintType::Json => Arc::new(Box::new(JsonOutput::new())),
            PrintType::Print => Arc::new(Box::new(PrintOutput {})),
            PrintType::Interactive => Arc::new(Box::new(InteractiveOutput::new())),
        },
        ..Default::default()
    };

    match &args.command {
        Commands::Download { tag } => {
            cmds::download(&rapid_store, &opts, tag).await;
        }
        Commands::DownloadSdp { sdp } => {
            cmds::download_sdp(&rapid_store, &opts, sdp).await;
        }
        Commands::DownloadRegistry => {
            cmds::download_registry(&rapid_store, &opts).await;
        }
        Commands::DownloadRepo { repo } => {
            cmds::download_repo(&rapid_store, &opts, repo.as_deref()).await;
        }

        Commands::CheckSdp { sdp } => {
            cmds::check_sdp(&rapid_store, sdp);
        }
        Commands::Validate { fullname } => {
            opts.metadata_source = MetadataSource::Local;
            cmds::validate_by_fullname(&rapid_store, &opts, fullname).await;
        }
        Commands::Fix { fullname } => {
            opts.metadata_source = MetadataSource::Local;
            cmds::fix(&rapid_store, &opts, fullname).await;
        }
    }
}
