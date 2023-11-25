use std::{
    env,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use bittorrent_starter_rust::{decode_bencoded_value, BenDecodeErrors};
use serde_json::json;
use sha1::{Digest, Sha1};

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

        let response = reqwest::blocking::Client::new()
            .get(info.trackter_url)
            .query(&[("a", "b")])
            .send()
            .unwrap();

        let body = response.text().unwrap();

        let iterator = &mut body.as_bytes().iter().copied();

        let value = decode_bencoded_value(iterator);

        print!("{}", value.unwrap());
    } else {
        println!("unknown command: {}", args[1])
    }
}
