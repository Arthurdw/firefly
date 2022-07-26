use std::collections::HashMap;

pub type Map = HashMap<String, (String, String)>;

// The byte that is used to represent the end of a partition. (field)
pub static DELIMITER_BYTE: u8 = 0x0;
/// The amount of elements that are in the object that should be serialized.
pub static PARTITION_AMOUNT: u8 = 3;

/// Check if a byte is ascii or if it is the delimiter byte.
///
/// Arguments
///
/// * `byte` - The byte to check.
pub fn process_byte(byte: u8) -> Result<u8, String> {
    if !byte.is_ascii() {
        return Err(format!("Non-ASCII character: {}", byte));
    } else if byte == DELIMITER_BYTE {
        return Err(format!(
            "Illigal action (using delimiter character): {}",
            byte
        ));
    }

    Ok(byte)
}

/// Check if the message is valid, convert it to bytes and prepend the
/// delimiter byte.
///
/// Arguments
///
/// * `message` - The message to check and convert
pub fn process_message(message: String) -> Result<Vec<u8>, String> {
    if message.is_empty() {
        return Err("Message is empty!".to_string());
    }

    let bytes = message.bytes();
    let mut result = Vec::with_capacity(bytes.len() + 1);
    result.push(DELIMITER_BYTE);

    for byte in bytes {
        let processed_byte = process_byte(byte)?;
        result.push(processed_byte);
    }

    Ok(result)
}

/// Serialize a Map of ascii strings into a collection of bytes.
///
/// Arguments
///
/// * `map` - A Map to serialize.
pub fn to_vec(map: Map) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();

    for (key, value) in map {
        let key_bytes = process_message(key)?;
        let value_bytes = process_message(value.0)?;
        let ttl_bytes = process_message(value.1)?;

        result.extend_from_slice(&key_bytes);
        result.extend_from_slice(&value_bytes);
        result.extend_from_slice(&ttl_bytes);
    }

    Ok(result)
}

/// Deserialize a slice of bytes into a Map.
/// Arguments
///
/// * `slice` - The slice of bytes to deserialize.
pub fn from_slice(slice: &[u8]) -> Result<Map, String> {
    // TODO: Refactor this and write tests
    let mut res: Map = HashMap::new();

    let mut current_partition: u8 = 0;
    let mut current_bytes: Vec<u8> = Vec::new();
    let mut current_representation: Vec<String> = Vec::with_capacity(PARTITION_AMOUNT as usize);

    for byte in slice.to_owned() {
        // This is the start of a new partition.
        if byte == DELIMITER_BYTE {
            if !current_bytes.is_empty() {
                current_representation.push(String::from_utf8(current_bytes).unwrap());
            }

            if current_partition == PARTITION_AMOUNT {
                res.insert(
                    current_representation.remove(0),
                    (
                        current_representation.remove(0),
                        current_representation.remove(0),
                    ),
                );

                current_partition = 0;
            } else {
                current_partition += 1;
            }

            current_bytes = Vec::new();

            continue;
        }
        current_bytes.push(byte);
    }

    Ok(res)
}
