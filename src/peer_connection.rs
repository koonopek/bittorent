use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub struct PeerConnection {
    pub tcp_stream: TcpStream,
    pub peer_id: String,
    pub extension_enabled: bool,
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
    Extended = 20,
}

impl PeerConnection {
    pub fn handshake(peer: &str, info_hash: &[u8], extension_enabled: bool) -> PeerConnection {
        println!("Connection to peer {}", peer);
        let mut stream = TcpStream::connect(peer).expect("Failed to connect to peer");

        let mut payload = Vec::with_capacity(68); // 28 + 20 + 20

        let magic_bytes = if extension_enabled {
            b"BitTorrent protocol\x00\x00\x00\x00\x00\x10\x00\x00"
        } else {
            b"BitTorrent protocol\x00\x00\x00\x00\x00\x00\x00\x00"
        };

        payload.push(19); // 1 byte
        payload.extend_from_slice(magic_bytes); // 27 bytes
        payload.extend_from_slice(info_hash); // 20 bytes
        payload.extend_from_slice(b"00112233445566778899"); // 20 bytes

        stream
            .write_all(&payload)
            .expect("Failed to write to tcp stream");

        let mut return_message_buf: [u8; 68] = [0; 68];
        stream
            .read_exact(&mut return_message_buf)
            .expect("Failed to read peer handshake response");

        // let received_magic_bytes = &return_message_buf[1..28];
        // assert_eq!(received_magic_bytes, magic_bytes);

        let info_hash_received = &return_message_buf[28..48];
        assert_eq!(info_hash_received, info_hash);

        let peer_id = hex::encode(&return_message_buf[48..68]);

        PeerConnection {
            tcp_stream: stream,
            peer_id,
            extension_enabled: return_message_buf[25] == 16,
        }
    }

    pub fn send_message(&mut self, message_type: MessageType, payload: Vec<u8>) {
        let payload_len: u32 = payload.len() as u32 + 1;

        let mut message_payload: Vec<u8> = Vec::with_capacity(4 + payload_len as usize);

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
            .expect("failed to read message size");

        let mut message_id_buf: [u8; 1] = [0; 1];
        self.tcp_stream
            .read_exact(&mut message_id_buf)
            .expect("Failed to read message id");

        let message_type = match message_id_buf[0] {
            1 => MessageType::Unchoked,
            5 => MessageType::BitField,
            7 => MessageType::Piece,
            20 => MessageType::Extended,
            id => panic!("Unknown message type {}", id),
        };

        let message = match message_type {
            MessageType::Extended => {
                let mut extended_message_id_buf: [u8; 1] = [0; 1];
                self.tcp_stream
                    .read_exact(&mut extended_message_id_buf)
                    .expect("Failed to read extended message id");

                let payload_size = match u32::from_be_bytes(payload_size_buf) {
                    x if x == 0 => 0 as usize,
                    x => (x - 2) as usize,
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
            MessageType::BitField | MessageType::Unchoked | MessageType::Piece => {
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
            _ => panic!("Don't know how to handle message type {:?}", message_type),
        };

        message
    }
}
