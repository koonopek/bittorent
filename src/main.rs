use std::{collections::HashMap, env, str::Chars};

use serde_json::{json, Map};

#[derive(Debug)]
enum BenDecodeErrors {
    StringDecodingError,
    MissingValueForDictKey,
    End,
    UknownError,
}

fn decode_bencoded_value(chars: &mut Chars) -> Result<serde_json::Value, BenDecodeErrors> {
    match chars.next() {
        // integer
        Some('i') => {
            let not_e = chars.take_while(|c| c != &'e').collect::<String>();

            return Ok(serde_json::Value::Number(
                not_e.parse::<i64>().unwrap().into(),
            ));
        }
        // string
        Some(c) if c.is_digit(10) => {
            let mut digits: String = chars.by_ref().take_while(|c| c != &':').collect();
            digits.insert(0, c);

            let length: usize = match digits.parse() {
                Ok(number) => number,
                _ => return Err(BenDecodeErrors::StringDecodingError),
            };

            let string: String = chars.take(length).collect();

            return Ok(serde_json::Value::String(string));
        }
        // list
        Some('l') => {
            let mut list = vec![];
            loop {
                match decode_bencoded_value(chars) {
                    Ok(value) => list.push(value),
                    Err(BenDecodeErrors::End) => return Ok(serde_json::Value::Array(list)),
                    _ => return Err(BenDecodeErrors::UknownError),
                };
            }
        }
        // dict
        Some('d') => {
            let mut dict: Map<String, serde_json::Value> = Map::new();
            loop {
                match decode_bencoded_value(chars) {
                    Ok(serde_json::Value::String(key)) => match decode_bencoded_value(chars) {
                        Ok(value) => dict.insert(key, value),
                        _ => return Err(BenDecodeErrors::MissingValueForDictKey),
                    },
                    Err(BenDecodeErrors::End) => return Ok(serde_json::Value::Object(dict)),
                    _ => return Err(BenDecodeErrors::UknownError),
                };
            }
        }
        // terminator
        Some('e') => {
            return Err(BenDecodeErrors::End);
        }
        _ => return Err(BenDecodeErrors::UknownError),
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(&mut encoded_value.chars()).unwrap();
        println!("{}", json!(decoded_value));
    } else {
        println!("unknown command: {}", args[1])
    }
}
