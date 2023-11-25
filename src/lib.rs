use std::{
    fmt::Display,
    fs,
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

pub fn get_metafile_info(args: &Vec<String>) -> MetaInfoFile {
    let file_path = &*args[2];
    let info = read_metainfo_file(&PathBuf::from(file_path)).unwrap();

    let announce = info["announce"].as_str().unwrap();
    let length = info["info"].as_object().unwrap()["length"]
        .as_u64()
        .unwrap();

    let bencoded_info = serde_bencode::to_bytes(&info["info"]).unwrap();
    let mut hasher = Sha1::new();
    hasher.update(&bencoded_info);
    let hash = hasher.finalize();

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
