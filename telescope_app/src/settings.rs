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

    if current_dir.join("data_dir.txt").exists() {
        let data_dir = std::fs::read_to_string(current_dir.join("data_dir.txt")).expect("Could not read data_dir.txt");
        let path_buf = PathBuf::from(data_dir);
        let create_status = std::fs::create_dir_all(path_buf.clone());
        if create_status.is_ok() {
            println!("Created data directory at {}", path_buf.display());
        }
        return path_buf;
    }
    
    current_dir
}

pub fn persist_new_data_directory(data_dir: PathBuf) {
    // in the current directory
    let current_dir = std::env::current_dir().expect("Could not get current directory");
    // write a new file called data_dir.txt
    let data_dir_file = current_dir.join("data_dir.txt");
    std::fs::write(data_dir_file, data_dir.to_str().unwrap()).expect("Could not write data_dir.txt");
}