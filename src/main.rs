use std::{env, hash};

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

        let info_hash_encoded: String = info.hash.iter().map(|&b| format!("%{:02X}", b)).collect();

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

        let body = response.text().unwrap();

        print!("body {}", body);

        let iterator = &mut body.as_bytes().iter().copied();

        let value = decode_bencoded_value(iterator);

        print!("{}", value.unwrap());
    } else {
        println!("unknown command: {}", args[1])
    }
}
