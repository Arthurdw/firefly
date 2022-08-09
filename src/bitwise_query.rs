#![allow(dead_code)]

use anyhow::{anyhow, Result};
use std::{string::FromUtf8Error, u8};

use crate::query::{parse_query_arguments, QueryType};

const DELIMITER: u8 = 0x0;

// TODO: Write some documentation
pub(crate) fn get_query_type(query: String) -> Option<QueryType> {
    let mut first_byte: Option<u8> = None;

    for byte in query.bytes() {
        if byte == '\n' as u8 || byte == ' ' as u8 {
            continue;
        }

        first_byte = Some(byte);
        break;
    }
    return match first_byte {
        Some(byte) => QueryType::from_byte(byte),
        None => None,
    };
}

// TODO: Write some documentation
pub(crate) fn get_arguments(
    query_type: QueryType,
    query: String,
) -> Result<Vec<String>, FromUtf8Error> {
    let mut arguments = Vec::new();
    let mut current_argument = Vec::new();

    let mut has_passed_identifier = false;

    for byte in query.bytes() {
        if !has_passed_identifier {
            has_passed_identifier = byte == query_type.as_byte();
            continue;
        }

        if byte == DELIMITER {
            arguments.push(String::from_utf8(current_argument)?);
            current_argument = Vec::new();
            continue;
        }

        current_argument.push(byte);
    }

    arguments.push(String::from_utf8(current_argument)?);

    Ok(arguments)
}

// TODO: Write some documentation
pub fn parse_query(query: String) -> Result<(QueryType, Vec<String>)> {
    match get_query_type(query.clone()) {
        Some(qt) => parse_query_arguments(|q| get_arguments(qt, q), qt, query),
        None => Err(anyhow!("Invalid query type")),
    }
}
