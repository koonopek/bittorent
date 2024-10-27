use sha1::{Digest, Sha1};

pub mod bencode;
pub mod discover_peers;
pub mod magnet_link;
pub mod meta_info_file;
pub mod peer_connection;
pub mod pieces;

pub fn sha1_it(bytes: &Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    let hash = hasher.finalize();
    hash.to_vec()
}
