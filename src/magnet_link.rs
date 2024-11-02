use std::collections::HashMap;

pub struct MagnetLink {
    pub tracker_url: String,
    pub hash: Vec<u8>,
    pub file_name: String,
}

pub fn parse_magnet_link_url(magnet_link_url: &str) -> MagnetLink {
    let (prefix, magnet_params_encoded) = magnet_link_url.split_once(":?").unwrap();

    assert_eq!(prefix, "magnet");

    // decode url
    let parse = url::form_urlencoded::parse(magnet_params_encoded.as_bytes());
    let decoded_url: HashMap<String, String> = parse.into_owned().collect();

    assert_eq!(decoded_url.len(), 3);

    let (xt_prefix, hash) = decoded_url.get("xt").expect("Expected xt").split_at(9);
    assert_eq!(xt_prefix, "urn:btih:");
    let tracker_url = decoded_url.get("tr").expect("Expected tr");
    let file_name = decoded_url.get("dn").expect("Expected file_name");

    MagnetLink {
        tracker_url: tracker_url.to_string(),
        hash: hex::decode(hash).expect("failed to parse hash"),
        file_name: file_name.to_string(),
    }
}
