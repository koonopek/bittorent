use std::{env, str::Chars};

use serde_json::json;

fn decode_bencoded_value(mut chars: Chars) -> Result<serde_json::Value, &str> {
    match chars.next() {
        // integer
        Some('i') => {
            let not_e = chars.take_while(|c| c != &'e').collect::<String>();
            // println!("{}", not_e);

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
                _ => return Err("Failed to convert string to number, when decoding string"),
            };

            let string: String = chars.take(length).collect();

            return Ok(serde_json::Value::String(string));
        }
        // list
        Some('l') => {
            let mut list = vec![];

            let list_content = chars.by_ref().take_while(|c| c != &'e').collect::<String>();

            while chars.next().is_some() {
                list.push(decode_bencoded_value(list_content.chars()));
            }

            // decode_bencoded_value(chars);
            return Ok(serde_json::Value::Array(vec![]));
        }
        _ => return Err(""),
    }
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value.chars());
        println!("{}", json!(decoded_value.unwrap()));
    } else {
        println!("unknown command: {}", args[1])
    }
}
