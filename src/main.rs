use std::{
    env, fs,
    path::{Path, PathBuf},
};

use bittorrent_starter_rust::{decode_bencoded_value, BenDecodeErrors};
use serde_json::json;
use sha1::{Digest, Sha1};

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

        let announce = info["announce"].as_str().unwrap();
        let length = info["info"].as_object().unwrap()["length"]
            .as_u64()
            .unwrap();

        let bencoded_info = serde_bencode::to_bytes(&info["info"]).unwrap();

        let mut hasher = Sha1::new();
        hasher.update(&bencoded_info);
        let hash = hasher.finalize();

        println!("Tracker URL: {}", announce);
        println!("Length: {}", length);
        println!("Info Hash: {}", hex::encode(hash));
    } else {
        println!("unknown command: {}", args[1])
    }
}
