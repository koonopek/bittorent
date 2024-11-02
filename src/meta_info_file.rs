use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    bencode::{self, decode_bencoded_value, BenDecodeErrors},
    magnet_link::MagnetLink,
    peer_connection::{MessageType, PeerConnection},
    sha1_it,
};

#[derive(Debug)]
pub struct MetaInfo {
    pub tracker_url: String,
    pub length: usize,
    pub hash: Vec<u8>,
    pub piece_length: usize,
    // FIXME: sub optimal this should be Vec<Vec<u8>>
    pub piece_hashes: Vec<String>,
    pub file_name: Option<String>,
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
            file_name: Option::None,
            piece_hashes: pieces,
        };
    }

    pub fn from_magnet_link(magnet_link: &MagnetLink, peer: &String) -> Self {
        let mut peer_connection = PeerConnection::handshake(peer, &magnet_link.hash, true);

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

        let response =
            bencode::decode_bencoded_value(&mut message.payload.clone().into_iter()).unwrap();
        // FIXME: here could be multiple pieces
        let piece_length = response["total_size"].as_i64().unwrap();

        let (_, piece_data) = message
            .payload
            .split_at(message.payload.len() - piece_length as usize);

        // FIXME: not optimal
        let piece_content =
            bencode::decode_bencoded_value(&mut piece_data.to_vec().into_iter()).unwrap();

        // dbg!(piece_content);
        let hash = sha1_it(&piece_data.to_vec());

        assert_eq!(&hash, &magnet_link.hash);

        return MetaInfo {
            tracker_url: magnet_link.tracker_url.to_owned(),
            length: piece_content["length"].as_i64().unwrap() as usize,
            hash: hash,
            file_name: Some(String::from(piece_content["name"].as_str().unwrap())),
            piece_length: piece_content["piece length"].as_i64().unwrap() as usize,
            piece_hashes: piece_content["pieces"]
                .as_str()
                .unwrap()
                .as_bytes()
                .chunks(20)
                .map(|x| hex::encode(x))
                .collect(),
        };
    }
}

fn read_metainfo_file(file_path: &Path) -> Result<serde_json::Value, BenDecodeErrors> {
    let content = fs::read(file_path).unwrap();

    return decode_bencoded_value(&mut content.into_iter());
}
