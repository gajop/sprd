use std::path;

use clap::{App, Arg};

mod api;
mod commands;
mod download;
mod gz;
mod rapid;
mod util;

#[tokio::main]
async fn main() {
    let matches = App::new("prd-rs")
        .version("0.1.0")
        .author("Gajo Petrovic <gajopetrovic@gmail.com>")
        .about("Rapid client")
        .arg(
            Arg::with_name("root-folder")
                .long("root-folder")
                .takes_value(true),
        )
        .subcommand(
            clap::SubCommand::with_name("check-sdp")
                .arg(
                    Arg::with_name("sdp")
                        .index(1)
                        .takes_value(true)
                        .required(true),
                )
                .help("Check if SDP is fully downloaded"),
        )
        .subcommand(
            clap::SubCommand::with_name("download")
                .arg(
                    Arg::with_name("tag")
                        .index(1)
                        .takes_value(true)
                        .required(true),
                )
                .help("Download the specified rapid tag"),
        )
        .subcommand(
            clap::SubCommand::with_name("download-registry").help("Download the registry metadata"),
        )
        .subcommand(
            clap::SubCommand::with_name("download-repo")
                .arg(
                    Arg::with_name("repo")
                        .index(1)
                        .takes_value(true)
                        .required(false),
                )
                .help("Download the repository metadata"),
        )
        .subcommand(
            clap::SubCommand::with_name("download-sdp")
                .arg(
                    Arg::with_name("sdp")
                        .index(1)
                        .takes_value(true)
                        .required(true),
                )
                .help("Download SDP"),
        )
        .get_matches();

    let root_folder = if matches.is_present("root-folder") {
        path::PathBuf::from(matches.value_of("root-folder").unwrap())
    } else {
        util::default_spring_dir()
    };
    let rapid_store = rapid::rapid_store::RapidStore {
        root_folder: &root_folder,
    };

    match matches.subcommand() {
        ("check-sdp", Some(sub_m)) => {
            let sdp_md5 = sub_m.value_of("sdp").unwrap();
            commands::check_sdp(&rapid_store, sdp_md5);
        }
        ("download", Some(sub_m)) => {
            let tag = sub_m.value_of("tag").unwrap();
            commands::download(&rapid_store, tag).await;
        }
        ("download-sdp", Some(sub_m)) => {
            let sdp_md5 = sub_m.value_of("sdp").unwrap();
            commands::download_sdp(&rapid_store, sdp_md5).await;
        }
        ("download-registry", Some(_)) => {
            commands::download_registry(&rapid_store).await;
        }
        ("download-repo", Some(sub_m)) => {
            let repo = sub_m.value_of("repo");
            commands::download_repo(&rapid_store, repo).await;
        }
        _ => println!("No subcommand was used"),
    }
}
