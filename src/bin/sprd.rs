#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

use std::{path::PathBuf, sync::Arc};

use clap::{ArgEnum, Parser, Subcommand};
use output::{interactive::InteractiveOutput, json::JsonOutput};
use sprd::{
    api::{DownloadOptions, MetadataSource},
    event::{PrintOutput, SilentOutput},
    rapid,
};

use atty::Stream;

mod cmds;
mod output;

#[derive(Parser)]
#[clap(author, version, about, long_about = "Rapid client")]
struct Args {
    #[clap(short, long)] // parse(from_os_str) needed?
    root: Option<PathBuf>,

    #[clap(short, long, arg_enum, default_value_t = OutputType::Auto)]
    output: OutputType,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum OutputType {
    Auto,
    Silent,
    Print,
    Json,
    Interactive,
}

#[derive(Subcommand)]
enum Commands {
    /// Download the specified (rapid) resource
    Download { rapid_name: String },

    /// Download the registry file
    MetaDownloadRegistry,
    /// Download the repository metadata file
    MetaDownloadRepo { rapid_repo: Option<String> },
    /// Download the sdp file
    MetaDownloadSdp { sdp: String },

    /// Check if fully downloaded
    CheckExists { rapid_name: String },
    /// Check if fully downloaded & valid
    Verify { rapid_name: String },
    /// Verify and fix any corruption
    Fix { rapid_name: String },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let rapid_store = match args.root {
        Some(path) => rapid::rapid_store::RapidStore::new(path),
        None => rapid::rapid_store::RapidStore::default(),
    };

    let mut opts = DownloadOptions {
        print: match args.output {
            OutputType::Silent => Arc::new(Box::new(SilentOutput {})),
            OutputType::Auto => {
                if atty::is(Stream::Stdout) {
                    Arc::new(Box::new(InteractiveOutput::new()))
                } else {
                    Arc::new(Box::new(PrintOutput {}))
                }
            }
            OutputType::Json => Arc::new(Box::new(JsonOutput::new())),
            OutputType::Print => Arc::new(Box::new(PrintOutput {})),
            OutputType::Interactive => Arc::new(Box::new(InteractiveOutput::new())),
        },
        ..Default::default()
    };

    match &args.command {
        Commands::Download {
            rapid_name: fullname,
        } => {
            cmds::download(&rapid_store, &opts, fullname).await;
        }
        Commands::MetaDownloadSdp { sdp } => {
            cmds::meta_download_sdp(&rapid_store, &opts, sdp).await;
        }
        Commands::MetaDownloadRegistry => {
            cmds::meta_download_registry(&rapid_store, &opts).await;
        }
        Commands::MetaDownloadRepo { rapid_repo: repo } => {
            cmds::meta_download_repo(&rapid_store, &opts, repo.as_deref()).await;
        }

        Commands::CheckExists {
            rapid_name: fullname,
        } => {
            cmds::check_exists(&rapid_store, fullname);
        }
        Commands::Verify {
            rapid_name: fullname,
        } => {
            opts.metadata_source = MetadataSource::Local;
            cmds::verify(&rapid_store, &opts, fullname).await;
        }
        Commands::Fix {
            rapid_name: fullname,
        } => {
            opts.metadata_source = MetadataSource::Local;
            cmds::fix(&rapid_store, &opts, fullname).await;
        }
    }
}
