use std::path::PathBuf;
use std::env;

pub fn resolve_user_data_directory() -> PathBuf {
    let current_dir = std::env::current_dir().expect("Could not get current directory");
    if std::env::var("DATA_DIR").is_ok() {
        if let Ok(data_dir) = std::env::var("DATA_DIR") {
            let path_buf = PathBuf::from(data_dir);
            std::fs::create_dir_all(path_buf.clone()).expect("Could not create data directory");
            return path_buf;
        }
    }

    if current_dir.join("portable.ini").exists() || current_dir.join(".portable").exists() {
        let config_dir = current_dir.join(".config");
        std::fs::create_dir_all(config_dir.clone()).expect("Could not create config directory in current directory for portable mode");
        return config_dir;
    }

    current_dir
}