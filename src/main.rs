use std::{
    env,
    fs::File,
    io::{IoSlice, Write},
    path::PathBuf,
};

use bittorrent_starter_rust::{
    bencode::decode_bencoded_value, discover_peers::discover_peers,
    magnet_link::parse_magnet_link_url, meta_info_file::MetaInfo, peer_connection::PeerConnection,
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
        let info = MetaInfo::from_path(&PathBuf::from(file_path));
        print!("{}", info);
    } else if command == "peers" {
        let info = MetaInfo::from_path(&PathBuf::from(file_path));
        let peers = discover_peers(&info.hash, info.piece_length, &info.tracker_url);
        println!("{:?}", peers);
    } else if command == "handshake" {
        let info = MetaInfo::from_path(&PathBuf::from(file_path));
        let peer = &args[3];

        let connection = PeerConnection::handshake(peer, &info.hash);
        println!("Handshaked with Peer ID: {}", connection.peer_id);
    } else if command == "download_piece" {
        let (save_to, torrent_info_path, piece_number) = (&args[3], &args[4], &args[5]);
        let piece_index: usize = piece_number.parse().expect("Failed to parse piece index");

        let info = MetaInfo::from_path(&PathBuf::from(torrent_info_path));
        let discover_peers = discover_peers(&info.hash, info.piece_length, &info.tracker_url);
        let peers = discover_peers;
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

        let info = MetaInfo::from_path(&PathBuf::from(torrent_info_path));
        let peers = discover_peers(&info.hash, info.piece_length, &info.tracker_url);
        println!("Peers {:?}", peers);

        let pieces_count = int_div_with_ceil(info.length, info.piece_length);

        let pieces_indexes: Vec<_> = (0..pieces_count).collect();
        let chunk_size: usize = int_div_with_ceil(pieces_count, peers.len());
        let chunks_piece_indexes = pieces_indexes.chunks(chunk_size);

        let jobs: Vec<_> = chunks_piece_indexes.zip(peers.into_iter()).collect();

        println!("Scheduled piece indexes to download per peer {:?}", jobs);

        let mut pieces: Vec<_> = jobs
            .into_par_iter()
            .map(|(indexes, peer)| {
                indexes
                    .into_iter()
                    .map(|piece_index| {
                        let result = download_piece(&peer, &info, *piece_index);
                        println!(
                            "Peer {} downloaded {}/{}",
                            peer,
                            piece_index + 1,
                            pieces_count
                        );
                        return result;
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        pieces.sort_unstable_by_key(|x| x.0);

        let io_vector: Vec<_> = pieces.iter().map(|x| IoSlice::new(&x.1)).collect();

        let mut file = File::create(save_to).expect("Failed to open file");
        file.write_vectored(&io_vector).unwrap();
        file.flush().expect("Failed to flush file");
        println!("Downloaded {} to {}.", torrent_info_path, save_to);
    } else if command == "magnet_parse" {
        let magnet_link = &args[2];

        let magnet_link = parse_magnet_link_url(magnet_link);

        println!("Tracker URL: {}", magnet_link.tracker_url);
        println!("Info Hash: {}", magnet_link.hash);
    } else if command == "magnet_info" {
        let magnet_link = &args[2];

        let magnet_link = parse_magnet_link_url(magnet_link);

        println!("Tracker URL: {}", magnet_link.tracker_url);
        println!("Info Hash: {}", magnet_link.hash);
    } else if command == "magnet_handshake" {
        let magnet_link = &args[2];

        MetaInfo::from_magnet_link_url(magnet_link);
    } else {
        println!("unknown command: {}", command)
    }
}

fn int_div_with_ceil(a: usize, b: usize) -> usize {
    match (a / b, a % b) {
        (count, reminder) if reminder > 0 => return count + 1,
        (count, _) => return count,
    };
}
