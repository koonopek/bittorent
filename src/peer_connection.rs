use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::{bencode::decode_bencoded_value, meta_info_file::MetaInfo};

pub struct PeerConnection {
    pub tcp_stream: TcpStream,
    pub peer_id: String,
}

pub struct Message {
    pub payload: Vec<u8>,
    pub message_type: MessageType,
}

#[derive(Debug, PartialEq)]
pub enum MessageType {
    Unchoked = 1,
    Interested = 2,
    BitField = 5,
    Request = 6,
    Piece = 7,
}

impl PeerConnection {
    pub fn handshake(peer: &str, info_hash: &[u8]) -> PeerConnection {
        println!("Connection to peer {}", peer);
        let mut stream = TcpStream::connect(peer).expect("Failed to connect to peer");

        let mut payload = Vec::with_capacity(68); // 28 + 20 + 20
        payload.push(19);
        // flipped bit means that we support metadata extension
        payload.extend_from_slice(b"BitTorrent protocol\x00\x00\x00\x00\x00\x10\x00\x00");
        payload.extend_from_slice(info_hash);
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

    pub fn send_message(&mut self, message_type: MessageType, payload: Vec<u8>) {
        let payload_len = payload.len() + 1;

        let mut message_payload: Vec<u8> = Vec::with_capacity(4 + payload_len);

        message_payload.extend_from_slice(&payload_len.to_be_bytes());
        message_payload.push(message_type as u8);
        message_payload.extend(payload);

        self.tcp_stream
            .write_all(&message_payload)
            .expect("Failed to write to tcp stream");
    }

    pub fn read_message(&mut self) -> Message {
        let mut payload_size_buf: [u8; 4] = [0; 4];
        self.tcp_stream
            .read_exact(&mut payload_size_buf)
            .expect("failed to reade message size");

        let mut message_id_buf: [u8; 1] = [0; 1];
        self.tcp_stream
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
        self.tcp_stream
            .read_exact(&mut payload)
            .expect("Failed to read buffer");

        Message {
            payload,
            message_type,
        }
    }
}
