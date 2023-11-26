use std::{
    env,
    io::{BufReader, Read, Write},
    net::TcpStream,
};

use bittorrent_starter_rust::{decode_bencoded_value, get_metafile_info};
use serde_json::json;
use sha1::digest::core_api::Buffer;

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
        let peers = discover_peers(info);
        println!("{:?}", peers);
    } else if command == "handshake" {
        let info = get_metafile_info(file_path);
        let peer = &args[3];

        println!("Connection to peer {}", peer);
        let mut stream = TcpStream::connect(peer).expect("Failed to connect to peer");

        let mut payload = Vec::with_capacity(69); //29 + 20 + 20
        payload.push(19);
        payload.extend_from_slice(b"BitTorrent protocol\x00\x00\x00\x00\x00\x00\x00\x00");
        payload.extend(info.hash);
        payload.extend_from_slice(b"00112233445566778899");

        stream
            .write_all(&payload)
            .expect("Failed to write to tcp stream");

        let mut buf_reader = BufReader::new(stream);

        let mut return_message: [u8; 69] = [0; 69];
        buf_reader
            .read_exact(&mut return_message)
            .expect("Failed to read peer handshake response");
    } else {
        println!("unknown command: {}", args[1])
    }
}

fn discover_peers(info: bittorrent_starter_rust::MetaInfoFile) -> Vec<String> {
    let info_hash_encoded: String = unsafe { String::from_utf8_unchecked(info.hash) };
    let response = reqwest::blocking::Client::new()
        .get(info.trackter_url)
        .query(&[
            ("info_hash", info_hash_encoded.as_str()),
            ("peer_id", "00112233445566778899"),
            ("port", "6881"),
            ("uploaded", "0"),
            ("downloaded", "0"),
            ("left", &info.length.to_string()),
            ("compact", "1"),
        ])
        .send()
        .unwrap();
    let body = response.bytes().unwrap();
    let iterator = &mut body.iter().copied();
    let value = decode_bencoded_value(iterator).unwrap();
    let encoded_peers = value.as_object().unwrap()["peers"]
        .as_str()
        .expect("peers can be parse to string")
        .as_bytes()
        .chunks(6);
    let mut peers = Vec::new();

    for encoded_peer in encoded_peers {
        let mut iterator = encoded_peer.iter();
        let first_octet = iterator.next().unwrap();
        let second_octet = iterator.next().unwrap();
        let third_octet = iterator.next().unwrap();
        let fourth_octet = iterator.next().unwrap();

        let first_byte_port = *iterator.next().unwrap() as u16;
        let second_byte_port = *iterator.next().unwrap() as u16;
        let port = (first_byte_port << 8) | second_byte_port;

        let peer_address = format!(
            "{}.{}.{}.{}:{}",
            first_octet, second_octet, third_octet, fourth_octet, port
        );
        peers.push(peer_address);
    }

    return peers;
}
