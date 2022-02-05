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
    let matches = App::new("sprd")
        .version("0.1.0")
        .author("Gajo Petrovic <gajopetrovic@gmail.com>")
        .about("Rapid client")
        .arg(
            Arg::new("root-folder")
                .long("root-folder")
                .takes_value(true),
        )
        .subcommand(
            App::new("check-sdp")
                .arg(Arg::new("sdp").index(1).takes_value(true).required(true))
                .about("Check if SDP is fully downloaded"),
        )
        .subcommand(
            App::new("download")
                .arg(Arg::new("tag").index(1).takes_value(true).required(true))
                .about("Download the specified rapid tag"),
        )
        .subcommand(App::new("download-registry").about("Download the registry metadata"))
        .subcommand(
            App::new("download-repo")
                .arg(Arg::new("repo").index(1).takes_value(true).required(false))
                .about("Download the repository metadata"),
        )
        .subcommand(
            App::new("download-sdp")
                .arg(Arg::new("sdp").index(1).takes_value(true).required(true))
                .about("Download SDP"),
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
        Some(("check-sdp", sub_m)) => {
            let sdp_md5 = sub_m.value_of("sdp").unwrap();
            commands::check_sdp(&rapid_store, sdp_md5);
        }
        Some(("download", sub_m)) => {
            let tag = sub_m.value_of("tag").unwrap();
            commands::download(&rapid_store, tag).await;
        }
        Some(("download-sdp", sub_m)) => {
            let sdp_md5 = sub_m.value_of("sdp").unwrap();
            commands::download_sdp(&rapid_store, sdp_md5).await;
        }
        Some(("download-registry", _)) => {
            commands::download_registry(&rapid_store).await;
        }
        Some(("download-repo", sub_m)) => {
            let repo = sub_m.value_of("repo");
            commands::download_repo(&rapid_store, repo).await;
        }
        _ => println!("No subcommand was used"),
    }
}
