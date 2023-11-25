use std::env;

use bittorrent_starter_rust::{decode_bencoded_value, get_metafile_info};
use serde_json::json;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // Uncomment this block to pass the first stage
        let mut encoded_value = args[2].bytes().into_iter();
        let decoded_value = decode_bencoded_value(&mut encoded_value).unwrap();
        println!("{}", json!(decoded_value));
    } else if command == "info" {
        let info = get_metafile_info(&args);
        print!("{}", info);
    } else if command == "peers" {
        let info = get_metafile_info(&args);

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

        let encoded_peers: Vec<_> = value.as_object().unwrap()["peers"]
            .as_array()
            .expect("peers are array?")
            .iter()
            .map(|x| x.as_str().unwrap().as_bytes())
            .collect();

        print!("{:?}", encoded_peers);
    } else {
        println!("unknown command: {}", args[1])
    }
}
