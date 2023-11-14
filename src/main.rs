use serde_json;
use std::env;

fn decode_bencoded_value(encoded_value: &str) -> Result<serde_json::Value, &str> {
    // If encoded_value starts with a digit, it's a number
    match encoded_value.split_once(":") {
        Some((count, value)) => {
            if value.len()
                != count
                    .parse::<usize>()
                    .expect("Supplied count cant be parsed to in")
            {
                return Err("Length missmatched");
            }
            return Ok(serde_json::Value::String(value.to_string()));
        }
        _ => return Err("Failed to decode bencoded"),
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.unwrap().to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
