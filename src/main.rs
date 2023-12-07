use std::{env, fs::File, io::Write};

use bittorrent_starter_rust::{
    decode_bencoded_value, discover_peers, download_piece, get_metafile_info, handshake,
};
use serde_json::json;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    let file_path = &args[2];

    if command == "decode" {
        // Uncomment this block to pass the first stage
        let mut encoded_value = args[2].bytes().into_iter();
        let decoded_value = decode_bencoded_value(&mut encoded_value).unwrap();
        println!("{}", json!(decoded_value));
    } else if command == "info" {
        let info = get_metafile_info(file_path);
        print!("{}", info);
    } else if command == "peers" {
        let info = get_metafile_info(file_path);
        let peers = discover_peers(&info);
        println!("{:?}", peers);
    } else if command == "handshake" {
        let info = get_metafile_info(file_path);
        let peer = &args[3];

        let connection = handshake(peer, &info);
        println!("Handshaked with Peer ID: {}", connection.peer_id);
    } else if command == "download_piece" {
        let (save_to, torrent_info_path, piece_number) = (&args[3], &args[4], &args[5]);
        let piece_index: usize = piece_number.parse().expect("Failed to parse piece index");

        let info = get_metafile_info(torrent_info_path);
        let peers = discover_peers(&info);
        println!("Peers {:?}", peers);
        let peer = peers
            .get((piece_index % 3) as usize)
            .expect("Expected at least one peer");
        println!("Selected peer {}", peer);

        let piece = download_piece(peer, &info, piece_index);

        let mut piece_content = File::create(save_to).expect("Failed to open file");
        println!("Piece {} downloaded to {}.", piece_index, save_to);
        piece_content.write(&piece).unwrap();
        piece_content.flush().expect("Failed to flush file");
    } else if command == "download" {
        let (save_to, torrent_info_path) = (&args[3], &args[4]);
    } else {
        println!("unknown command: {}", args[1])
    }
}
