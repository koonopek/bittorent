use std::{
    env,
    fmt::Display,
    fs,
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
        let info = get_metafile_info(&args);
        print!("{}", info);
    } else if command == "peers" {
        let info = get_metafile_info(&args);

        let response = reqwest::blocking::Client::new()
            .get(info.trackter_url)
            .query(&[("a", "b")])
            .send()
            .unwrap();

        let body = response.text().unwrap();

        let iterator = &mut body.as_bytes().iter().copied();

        let value = decode_bencoded_value(iterator);

        print!("{}", value.unwrap());
    } else {
        println!("unknown command: {}", args[1])
    }
}

struct MetaInfoFile {
    trackter_url: String,
    length: usize,
    hash: String,
    piece_length: usize,
    piece_hashes: Vec<String>,
}

impl Display for MetaInfoFile {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("Tracker URL: {}", self.trackter_url);
        println!("Length: {}", self.length);
        println!("Info Hash: {}", self.hash);
        println!("Piece Length: {}", self.piece_length);
        println!("Piece Hashes");
        for piece in &self.piece_hashes {
            println!("{}", piece);
        }
        Ok(())
    }
}

fn get_metafile_info(args: &Vec<String>) -> MetaInfoFile {
    let file_path = &*args[2];
    let info = read_metainfo_file(&PathBuf::from(file_path)).unwrap();

    let announce = info["announce"].as_str().unwrap();
    let length = info["info"].as_object().unwrap()["length"]
        .as_u64()
        .unwrap();

    let bencoded_info = serde_bencode::to_bytes(&info["info"]).unwrap();
    let mut hasher = Sha1::new();
    hasher.update(&bencoded_info);
    let hash = hasher.finalize();

    let piece_length = info["info"].as_object().unwrap()["piece length"]
        .as_u64()
        .unwrap();

    let pieces: Vec<_> = info["info"].as_object().unwrap()["pieces"]
        .as_str()
        .unwrap()
        .as_bytes()
        .chunks(20)
        .map(|x| hex::encode(x))
        .collect();

    return MetaInfoFile {
        trackter_url: announce.to_string(),
        length: length as usize,
        hash: hex::encode(hash),
        piece_length: piece_length as usize,
        piece_hashes: pieces,
    };
}
