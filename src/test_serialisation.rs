use std::collections::HashMap;

use crate::serialisation::{
    process_byte, process_message, to_vec, DELIMITER_BYTE, PARTITION_AMOUNT,
};

#[test]
fn test_process_byte() {
    assert_eq!(
        process_byte(DELIMITER_BYTE),
        Err(format!(
            "Illigal action (using delimiter character): {}",
            DELIMITER_BYTE
        ))
    );
    assert_eq!(
        process_byte(255),
        Err("Non-ASCII character: 255".to_string())
    );
    assert_eq!(process_byte('a' as u8), Ok('a' as u8));
}

#[test]
fn test_process_message() {
    let message = "Hello, world!";
    let bytes = process_message(message.clone().to_string()).unwrap();

    assert_eq!(bytes[0], DELIMITER_BYTE);
    assert_eq!(bytes.len(), message.len() + 1);
    assert_eq!(std::str::from_utf8(&bytes[1..]).unwrap(), message);
}

#[test]
fn test_to_vec_empty() {
    let map = HashMap::new();
    let bytes = to_vec(map).unwrap();

    assert_eq!(bytes.len(), 0);
}

#[test]
fn test_to_vec_one_key() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), ("value".to_string(), "1".to_string()));
    let bytes = to_vec(map).unwrap();

    assert_eq!(
        bytes.len(),
        PARTITION_AMOUNT as usize + "key".len() + "value".len() + "1".len()
    );
}
