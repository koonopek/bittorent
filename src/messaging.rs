use std::io::{Read, Write};

use crate::peers::PeerConnection;

pub struct Message {
    pub payload: Vec<u8>,
    pub message_type: MessageType,
}

#[derive(Debug, PartialEq)]
pub enum MessageType {
    Unchoked = 1,
    Intrested = 2,
    BitField = 5,
    Request = 6,
    Piece = 7,
}

pub fn read_message(connection: &mut PeerConnection) -> Message {
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

pub fn send_message(connection: &mut PeerConnection, message_type: MessageType, payload: Vec<u8>) {
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
