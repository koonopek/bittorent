use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    bencode::{decode_bencoded_value, BenDecodeErrors},
    discover_peers::discover_peers,
    magnet_link::parse_magnet_link_url,
    peer_connection::PeerConnection,
    sha1_it,
};

pub struct MetaInfo {
    pub tracker_url: String,
    pub length: usize,
    pub hash: Vec<u8>,
    pub piece_length: usize,
    pub piece_hashes: Vec<String>,
}

impl Display for MetaInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("Tracker URL: {}", self.tracker_url);
        println!("Length: {}", self.length);
        println!("Info Hash: {}", hex::encode(&self.hash));
        println!("Piece Length: {}", self.piece_length);
        println!("Piece Hashes");
        for piece in &self.piece_hashes {
            write!(f, "{}", piece)?;
        }
        Ok(())
    }
}

impl MetaInfo {
    pub fn from_path(file_path: &Path) -> Self {
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

        return MetaInfo {
            tracker_url: announce.to_string(),
            length: length as usize,
            hash: hash.to_vec(),
            piece_length: piece_length as usize,
            piece_hashes: pieces,
        };
    }

    pub fn from_magnet_link_url(magnet_link_url: &str) -> Self {
        let magnet_link = parse_magnet_link_url(magnet_link_url);

        let hash_bytes = hex::decode(&magnet_link.hash).expect("failed to decode magnet link hash");

        // some random number, because we don't know the length before fetching metadata
        let peers = discover_peers(&hash_bytes, 999, &magnet_link.tracker_url);

        let peer = peers.first().unwrap();

        let peer_connection = PeerConnection::handshake(peer, &hash_bytes);

        println!("Peer ID: {}", peer_connection.peer_id);

        return MetaInfo {
            tracker_url: String::new(),
            length: 0,
            hash: Vec::new(),
            piece_length: 0,
            piece_hashes: Vec::new(),
        };
    }
}

fn read_metainfo_file(file_path: &Path) -> Result<serde_json::Value, BenDecodeErrors> {
    let content = fs::read(file_path).unwrap();

    return decode_bencoded_value(&mut content.into_iter());
}
