use serde_json::Map;

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
