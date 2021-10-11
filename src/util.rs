use std::path;

use dirs::home_dir;

pub fn default_spring_dir() -> path::PathBuf {
    if let Some(home_dir) = home_dir() {
        let mut spring_dir = home_dir;
        if cfg!(windows) {
            spring_dir.push("My Games");
            spring_dir.push("Spring");
        } else if cfg!(unix) {
            spring_dir.push(".spring");
        }
        spring_dir
    } else {
        path::PathBuf::from(".")
    }
}
