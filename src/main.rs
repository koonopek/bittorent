use std::{
    env, fs,
    path::{Path, PathBuf},
};

use bittorrent_starter_rust::{decode_bencoded_value, BenDecodeErrors};
use serde_json::json;

fn read_metainfo_file(file_path: &Path) -> Result<serde_json::Value, BenDecodeErrors> {
    let content = fs::read(file_path).unwrap();

    return decode_bencoded_value(&mut content.into_iter());
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // Uncomment this block to pass the first stage
        let mut encoded_value = args[2].bytes().into_iter();
        let decoded_value = decode_bencoded_value(&mut encoded_value).unwrap();
        println!("{}", json!(decoded_value));
    } else if command == "info" {
        let file_path = &args[2];
        let info = read_metainfo_file(&PathBuf::from(file_path)).unwrap();

        print!("{}", info);

        let announce = info["announce"].as_str().unwrap();
        let length = info["info"].as_str().unwrap();

        print!("Tracker URL: {} Length: ${}", announce, length);
    } else {
        println!("unknown command: {}", args[1])
    }
}
