use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::{bencode::decode_bencoded_value, MetaInfoFile};

pub struct PeerConnection {
    pub tcp_stream: TcpStream,
    pub peer_id: String,
}

pub fn handshake(peer: &str, info: &MetaInfoFile) -> PeerConnection {
    println!("Connection to peer {}", peer);
    let mut stream = TcpStream::connect(peer).expect("Failed to connect to peer");

    let mut payload = Vec::with_capacity(68); // 28 + 20 + 20
    payload.push(19);
    payload.extend_from_slice(b"BitTorrent protocol\x00\x00\x00\x00\x00\x00\x00\x00");
    payload.extend_from_slice(&info.hash);
    payload.extend_from_slice(b"00112233445566778899");

    stream
        .write_all(&payload)
        .expect("Failed to write to tcp stream");

    let mut return_message_buf: [u8; 68] = [0; 68];
    stream
        .read_exact(&mut return_message_buf)
        .expect("Failed to read peer handshake response");

    let peer_id = hex::encode(&return_message_buf[48..68]);

    PeerConnection {
        tcp_stream: stream,
        peer_id,
    }
}

pub fn discover_peers(info: &MetaInfoFile) -> Vec<String> {
    let info_hash_encoded: String = unsafe { String::from_utf8_unchecked(info.hash.to_vec()) };
    let response = reqwest::blocking::Client::new()
        .get(&info.trackter_url)
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
        let peer_address = parse_peer_address(encoded_peer);
        peers.push(peer_address);
    }

    return peers;
}

fn parse_peer_address(encoded_peer: &[u8]) -> String {
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
    peer_address
}
