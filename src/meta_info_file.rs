use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    bencode::{self, decode_bencoded_value, BenDecodeErrors},
    discover_peers::discover_peers,
    magnet_link::parse_magnet_link_url,
    peer_connection::{MessageType, PeerConnection},
    sha1_it,
};

pub struct MetaInfo {
    pub tracker_url: String,
    pub length: usize,
    pub hash: Vec<u8>,
    pub piece_length: usize,
    pub piece_hashes: Vec<String>,
}

impl Display for MetaInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("Tracker URL: {}", self.tracker_url);
        println!("Length: {}", self.length);
        println!("Info Hash: {}", hex::encode(&self.hash));
        println!("Piece Length: {}", self.piece_length);
        println!("Piece Hashes");
        for piece in &self.piece_hashes {
            write!(f, "{}", piece)?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct MetadataHandshakePayload {
    ut_metadata: i32,
}

#[derive(Serialize, Deserialize)]
struct MetadataHandshakePayloadEnvelope {
    m: MetadataHandshakePayload,
}

#[derive(Serialize, Deserialize)]
struct MetadataMessagePayload {
    msg_type: i32,
    piece: i32,
}

impl MetaInfo {
    pub fn from_path(file_path: &Path) -> Self {
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

        return MetaInfo {
            tracker_url: announce.to_string(),
            length: length as usize,
            hash: hash.to_vec(),
            piece_length: piece_length as usize,
            piece_hashes: pieces,
        };
    }

    pub fn from_magnet_link_url(magnet_link_url: &str) -> Self {
        let magnet_link = parse_magnet_link_url(magnet_link_url);

        let hash_bytes = hex::decode(&magnet_link.hash).expect("failed to decode magnet link hash");

        // some random number, because we don't know the length before fetching metadata
        let peers = discover_peers(&hash_bytes, 999, &magnet_link.tracker_url);

        let peer = peers.first().unwrap();

        let mut peer_connection = PeerConnection::handshake(peer, &hash_bytes, true);

        let message = peer_connection.read_message();
        assert_eq!(message.message_type, MessageType::BitField);

        // we could choose different peer
        assert_eq!(peer_connection.extension_enabled, true);

        let payload = MetadataHandshakePayloadEnvelope {
            m: MetadataHandshakePayload { ut_metadata: 1 },
        };

        let mut handshake_message = vec![0; 1];
        handshake_message.extend_from_slice(&serde_bencode::to_bytes(&payload).unwrap());
        peer_connection.send_message(MessageType::Extended, handshake_message);

        let message = peer_connection.read_message();
        assert_eq!(message.message_type, MessageType::Extended);

        let payload = decode_bencoded_value(&mut message.payload.into_iter()).unwrap();

        println!("Peer ID: {}", peer_connection.peer_id);
        println!(
            "Peer Metadata Extension ID: {}",
            payload["m"]["ut_metadata"]
        );

        let peer_extension_id = &payload["m"]["ut_metadata"];

        let peer_extension_id: u8 = match peer_extension_id {
            serde_json::Value::Number(n) => u8::try_from(n.as_i64().unwrap()).unwrap(),
            _ => panic!("wrong value"),
        };

        let mut payload: Vec<u8> = vec![peer_extension_id; 1];
        payload.extend(
            serde_bencode::to_bytes(&MetadataMessagePayload {
                msg_type: 0,
                piece: 0,
            })
            .unwrap(),
        );
        peer_connection.send_message(MessageType::Extended, payload);

        let message = peer_connection.read_message();
        assert_eq!(message.message_type, MessageType::Extended);

        return MetaInfo {
            tracker_url: magnet_link.tracker_url.clone(),
            length: 0,
            hash: hash_bytes.to_vec(),
            piece_length: 0,
            piece_hashes: Vec::new(),
        };
    }
}

fn read_metainfo_file(file_path: &Path) -> Result<serde_json::Value, BenDecodeErrors> {
    let content = fs::read(file_path).unwrap();

    return decode_bencoded_value(&mut content.into_iter());
}
