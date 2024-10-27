use crate::bencode::decode_bencoded_value;

pub fn discover_peers(info_hash: &[u8], left: usize, tracker_url: &str) -> Vec<String> {
    let info_hash_encoded: String = unsafe { String::from_utf8_unchecked(info_hash.to_vec()) };
    let response = reqwest::blocking::Client::new()
        .get(tracker_url)
        .query(&[
            ("info_hash", info_hash_encoded.as_str()),
            ("peer_id", "00112233445566778899"),
            ("port", "6881"),
            ("uploaded", "0"),
            ("downloaded", "0"),
            ("left", &left.to_string()),
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
