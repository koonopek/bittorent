use std::{
    env,
    fs::File,
    io::{IoSlice, Write},
};

use bittorrent_starter_rust::{
    bencode::decode_bencoded_value,
    get_metafile_info,
    peers::{discover_peers, handshake},
    pieces::download_piece,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
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

        let piece = download_piece(peer, &info, piece_index).1;

        let mut file = File::create(save_to).expect("Failed to open file");
        file.write(&piece).unwrap();
        file.flush().expect("Failed to flush file");
        println!("Piece {} downloaded to {}.", piece_index, save_to);
    } else if command == "download" {
        let (save_to, torrent_info_path) = (&args[3], &args[4]);

        let info = get_metafile_info(torrent_info_path);
        let peers = &discover_peers(&info);
        println!("Peers {:?}", peers);

        let pieces_count = match (
            info.length / info.piece_length,
            info.length % info.piece_length,
        ) {
            (count, reminder) if reminder > 0 => count + 1,
            (count, _) => count,
        };

        let pieces_indexes: Vec<_> = (0..pieces_count).zip(peers).collect();

        let mut pieces: Vec<_> = pieces_indexes
            .into_par_iter()
            .map(|(piece_index, peer)| {
                let result = download_piece(peer, &info, piece_index);
                println!(
                    "{peer} downloaded piece {}/{}",
                    piece_index + 1,
                    pieces_count
                );
                return result;
            })
            .collect();

        pieces.sort_unstable_by_key(|x| x.0);

        let io_vector: Vec<_> = pieces.iter().map(|x| IoSlice::new(&x.1)).collect();

        let mut file = File::create(save_to).expect("Failed to open file");
        file.write_vectored(&io_vector).unwrap();
        file.flush().expect("Failed to flush file");
        println!("Downloaded {} to {}.", torrent_info_path, save_to);
    } else {
        println!("unknown command: {}", args[1])
    }
}
