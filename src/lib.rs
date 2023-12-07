use std::{
    cmp,
    fmt::Display,
    fs::{self},
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
};

use serde_json::Map;
use sha1::{Digest, Sha1};

#[derive(Debug)]
pub enum BenDecodeErrors {
    StringDecodingError,
    MissingValueForDictKey,
    End,
    DictError,
    ListError,
    UnexepctedChar,
}

#[derive(Debug, PartialEq)]
enum MessageType {
    Unchoked = 1,
    Intrested = 2,
    BitField = 5,
    Request = 6,
    Piece = 7,
}

pub fn decode_bencoded_value(
    chars: &mut impl Iterator<Item = u8>,
) -> Result<serde_json::Value, BenDecodeErrors> {
    match chars.next() {
        // integer
        Some(b'i') => {
            let not_e: String =
                String::from_utf8(chars.take_while(|c| c != &b'e').collect()).unwrap();

            return Ok(serde_json::Value::Number(
                not_e.parse::<i64>().unwrap().into(),
            ));
        }
        // string
        Some(c) if c.is_ascii_digit() => {
            let mut digits: String =
                String::from_utf8(chars.by_ref().take_while(|c| c != &b':').collect()).unwrap();
            digits.insert(0, c as char);

            let length: usize = match digits.parse() {
                Ok(number) => number,
                _ => return Err(BenDecodeErrors::StringDecodingError),
            };

            let string: String =
                unsafe { String::from_utf8_unchecked(chars.take(length).collect()) };
            return Ok(serde_json::Value::String(string));
        }
        // list
        Some(b'l') => {
            let mut list = vec![];
            loop {
                match decode_bencoded_value(chars) {
                    Ok(value) => list.push(value),
                    Err(BenDecodeErrors::End) => return Ok(serde_json::Value::Array(list)),
                    _ => return Err(BenDecodeErrors::ListError),
                };
            }
        }
        // dict
        Some(b'd') => {
            let mut dict: Map<String, serde_json::Value> = Map::new();
            loop {
                match decode_bencoded_value(chars) {
                    Ok(serde_json::Value::String(key)) => match decode_bencoded_value(chars) {
                        Ok(value) => dict.insert(key, value),
                        _ => return Err(BenDecodeErrors::MissingValueForDictKey),
                    },
                    Err(BenDecodeErrors::End) => return Ok(serde_json::Value::Object(dict)),
                    e => {
                        println!("Dict error");
                        println!("{:?}", e);
                        return Err(BenDecodeErrors::DictError);
                    }
                };
            }
        }
        // terminator
        Some(b'e') => {
            return Err(BenDecodeErrors::End);
        }
        w => {
            println!("unexpected char");
            println!("{:?}", w);
            return Err(BenDecodeErrors::UnexepctedChar);
        }
    }
}

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

pub fn download_piece(peer: &str, info: &MetaInfoFile, piece_index: usize) -> Vec<u8> {
    let mut connection = handshake(peer, &info);

    assert_eq!(
        read_message(&mut connection).message_type,
        MessageType::BitField
    );

    send_message(&mut connection, MessageType::Intrested, vec![]);

    assert_eq!(
        read_message(&mut connection).message_type,
        MessageType::Unchoked
    );

    let mut chunks_read = 0;

    let length_to_read = cmp::min(
        info.length - (piece_index * info.piece_length),
        info.piece_length,
    );

    loop {
        let current_chunk_to_read: i64 = length_to_read as i64 - (16 * 1024 * chunks_read) as i64;
        match current_chunk_to_read {
            x if x < 0 => {
                println!("End of read {}", chunks_read);
                break;
            }
            x if x >= 16 * 1024 => {
                println!("Full read {}", chunks_read);
                request_piece_part(
                    &mut connection,
                    piece_index as u32,
                    chunks_read as u32,
                    16 * 1024,
                )
            }
            x => {
                println!("Part read {}", chunks_read);
                request_piece_part(
                    &mut connection,
                    piece_index as u32,
                    chunks_read as u32,
                    x as u32,
                )
            }
        }
        chunks_read += 1;
    }

    let mut piece = Vec::with_capacity(info.piece_length);
    for _ in 0..chunks_read {
        let message = read_message(&mut connection);

        if message.message_type == MessageType::Piece {
            // let piece_index = u32::from_be_bytes(message.payload[0..4].try_into().unwrap());
            // let offset = u32::from_be_bytes(message.payload[4..8].try_into().unwrap());
            piece.extend_from_slice(&message.payload[8..])
        }
    }

    println!(
        "Checking piece hash {} == {}",
        info.piece_hashes[piece_index],
        hex::encode(sha1_it(&piece))
    );

    assert_eq!(info.piece_hashes[piece_index], hex::encode(sha1_it(&piece)));

    connection
        .tcp_stream
        .shutdown(std::net::Shutdown::Both)
        .expect("Failed to close tcp stream");

    piece
}

fn request_piece_part(
    connection: &mut PeerConnection,
    piece_index: u32,
    offset_block: u32,
    bytes_to_read: u32,
) {
    let begin: u32 = offset_block * 16 * 1024;
    println!(
        "Requesting piece {} begin {} length {}",
        piece_index, begin, bytes_to_read
    );

    let mut payload = Vec::with_capacity(12);
    payload.extend_from_slice(&piece_index.to_be_bytes());
    payload.extend_from_slice(&begin.to_be_bytes());
    payload.extend_from_slice(&bytes_to_read.to_be_bytes());

    send_message(connection, MessageType::Request, payload);
}

fn send_message(connection: &mut PeerConnection, message_type: MessageType, payload: Vec<u8>) {
    let payload_len = payload.len() + 1;

    let mut message_payload: Vec<u8> = Vec::with_capacity(4 + payload_len);

    message_payload.extend_from_slice(&payload_len.to_be_bytes());
    message_payload.push(message_type as u8);
    message_payload.extend(payload);

    connection
        .tcp_stream
        .write_all(&message_payload)
        .expect("Failed to write to tcp stream");
}

struct Message {
    payload: Vec<u8>,
    message_type: MessageType,
}

fn read_message(connection: &mut PeerConnection) -> Message {
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
        1 => MessageType::Unchoked,
        5 => MessageType::BitField,
        7 => MessageType::Piece,
        id => panic!("Unknown message type {}", id),
    };

    let payload_size = match u32::from_be_bytes(payload_size_buf) {
        x if x == 0 => 0 as usize,
        x => (x - 1) as usize,
    };

    let mut payload = vec![0; payload_size];
    connection
        .tcp_stream
        .read_exact(&mut payload)
        .expect("Failed to read buffer");

    Message {
        payload,
        message_type,
    }
}
