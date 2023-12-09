use std::{
    fmt::Display,
    fs::{self},
    path::{Path, PathBuf},
};

pub mod bencode;
pub mod messaging;
pub mod peers;
pub mod pieces;

use bencode::*;
use sha1::{Digest, Sha1};

pub struct MetaInfoFile {
    pub trackter_url: String,
    pub length: usize,
    pub hash: Vec<u8>,
    pub piece_length: usize,
    pub piece_hashes: Vec<String>,
}

impl Display for MetaInfoFile {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("Tracker URL: {}", self.trackter_url);
        println!("Length: {}", self.length);
        println!("Info Hash: {}", hex::encode(&self.hash));
        println!("Piece Length: {}", self.piece_length);
        println!("Piece Hashes");
        for piece in &self.piece_hashes {
            println!("{}", piece);
        }
        Ok(())
    }
}

fn read_metainfo_file(file_path: &Path) -> Result<serde_json::Value, BenDecodeErrors> {
    let content = fs::read(file_path).unwrap();

    return decode_bencoded_value(&mut content.into_iter());
}

pub fn get_metafile_info(file_path: &String) -> MetaInfoFile {
    let info = read_metainfo_file(&PathBuf::from(file_path)).unwrap();

    let announce = info["announce"].as_str().unwrap();
    let length = info["info"].as_object().unwrap()["length"]
        .as_u64()
        .unwrap();

    let bencoded_info = serde_bencode::to_bytes(&info["info"]).unwrap();
    let hash = sha1_it(&bencoded_info);

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
        hash: hash.to_vec(),
        piece_length: piece_length as usize,
        piece_hashes: pieces,
    };
}

pub fn sha1_it(bytes: &Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    let hash = hasher.finalize();
    hash.to_vec()
}
