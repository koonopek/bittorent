use std::{
    env,
    io::{Read, Write},
};

use bittorrent_starter_rust::{
    decode_bencoded_value, discover_peers, get_metafile_info, handshake,
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
        let (param_name, save_to, torrent_info_path, piece_number) =
            (&args[2], &args[3], &args[4], &args[5]);

        if save_to != "-o" {
            panic!("Expected -o in download_piece command")
        }

        let info = get_metafile_info(torrent_info_path);
        let peers = discover_peers(&info);
        let peer = peers.get(0).expect("Expected at least one peer");
        let mut connection = handshake(peer, &info);

        let mut payload_size_buf: [u8; 4] = [0; 4];
        connection
            .tcp_stream
            .read_exact(&mut payload_size_buf)
            .expect("failed to reade message size");

        let mut message_id_buf: [u8; 1] = [0; 1];
        connection
            .tcp_stream
            .read_exact(&mut message_id_buf)
            .expect("Failed to read message id");

        let message_type = match message_id_buf[0] {
            5 => MessageType::BitField,
            id => panic!("Unknown message type {}", id),
        };
    } else {
        println!("unknown command: {}", args[1])
    }
}

enum MessageType {
    BitField,
}
